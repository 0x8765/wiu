#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod platform;
mod decode;
mod svg;
use eframe::egui::{self, *};
use std::{path::PathBuf, sync::mpsc, time::{Duration, Instant}};

pub const EXTENSIONS: &[&str] = &["jpg","jpeg","jfif","png","bmp","tif","tiff","ico","gif","webp","avif","svg","pnm","ppm","pgm","pbm","tga"];
static DECODE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
static REQUEST: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
fn supported(p: &std::path::Path) -> bool { p.extension().is_some_and(|s| EXTENSIONS.contains(&s.to_string_lossy().to_ascii_lowercase().as_str())) }
fn main() -> eframe::Result {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.first().is_some_and(|s|s=="--check") {
        let mut failed=args.len()<2;
        for path in &args[1..] {match decode::load(std::path::Path::new(path)){Ok(p)=>println!("OK {} : {}x{}, {} frame(s)",path.to_string_lossy(),p.width,p.height,p.frames.len()),Err(e)=>{eprintln!("ERREUR {} : {e}",path.to_string_lossy());failed=true;}}}
        std::process::exit(if failed{1}else{0});
    }
    if args.first().is_some_and(|s| s == "--install" || s == "--uninstall") {
        let result = if args[0] == "--install" { platform::install() } else { platform::uninstall() };
        platform::message(&result.unwrap_or_else(|e| format!("Erreur : {e}")));
        return Ok(());
    }
    if args.first().is_some_and(|s| s == "--help") { platform::message("wiu [image ...]\nSans argument : choisir une image.\n--install : intégrer à l’Explorateur\n--uninstall : retirer l’intégration\nCtrl+O : ouvrir ; flèches : naviguer ; F11 : plein écran ; 0 : adapter ; 1 : 100 % ; espace : pause GIF."); return Ok(()); }
    let paths = args.into_iter().map(PathBuf::from).collect();
    eframe::run_native("wiu", eframe::NativeOptions { viewport: ViewportBuilder::default().with_inner_size([1080.,760.]).with_min_inner_size([480.,320.]).with_maximized(true).with_icon(icon()), ..Default::default() }, Box::new(|cc| {
        cc.egui_ctx.set_visuals(Visuals::dark());
        Ok(Box::new(Viewer::new(paths)))
    }))
}
fn icon() -> IconData {
    let mut rgba = vec![0; 32*32*4];
    for y in 0..32 { for x in 0..32 { let i=(y*32+x)*4; let c=if y>12 && y<27 && x>4 && x<28 && (y>26-x/2 || y>10+x/3) {[103,216,190,255]} else {[27,33,43,255]}; rgba[i..i+4].copy_from_slice(&c); } }
    IconData { rgba, width:32,height:32 }
}
fn navigation_button(ui: &mut Ui, forward: bool) -> Response {
    let label = if forward { "Image suivante" } else { "Image précédente" };
    let response = ui.add(Button::new("").min_size(vec2(30., ui.spacing().interact_size.y)));
    let center = response.rect.center();
    let direction = if forward { 1. } else { -1. };
    let stroke = ui.style().interact(&response).fg_stroke;
    ui.painter().add(Shape::line(vec![
        center + vec2(-direction * 3., -5.),
        center + vec2(direction * 3., 0.),
        center + vec2(-direction * 3., 5.),
    ], Stroke::new(1.8, stroke.color)));
    response.on_hover_text(label)
}
fn offset_after_zoom(offset: Vec2, canvas_center: Pos2, cursor: Pos2, ratio: f32) -> Vec2 {
    offset + (cursor - (canvas_center + offset)) * (1. - ratio)
}
struct Viewer {
    paths: Vec<PathBuf>, index: usize, initial: bool,
    receiver: Option<mpsc::Receiver<Result<decode::Picture,String>>>, picture: Option<decode::Picture>,
    texture: Option<TextureHandle>, frame: usize, deadline: Instant, paused: bool,
    zoom: Option<f32>, offset: Vec2, error: Option<String>, fullscreen: bool, installation_action: Option<&'static str>,
}
impl Viewer {
    fn new(paths:Vec<PathBuf>) -> Self { Self {paths,index:0,initial:true,receiver:None,picture:None,texture:None,frame:0,deadline:Instant::now(),paused:false,zoom:None,offset:Vec2::ZERO,error:None,fullscreen:false,installation_action:platform::installation_action()} }
    fn open(&mut self, mut paths: Vec<PathBuf>, ctx:&Context) {
        if paths.is_empty() { return; }
        self.index=0;
        if paths.len()==1 {
            let selected=std::fs::canonicalize(&paths[0]).unwrap_or(paths[0].clone());
            if let Some(parent)=selected.parent() {
                let mut siblings:Vec<_>=std::fs::read_dir(parent).into_iter().flatten().filter_map(Result::ok).map(|e|e.path()).filter(|p|p.is_file()&&supported(p)).collect();
                siblings.sort_by_cached_key(|p|p.file_name().unwrap_or_default().to_string_lossy().to_lowercase());
                if let Some(index)=siblings.iter().position(|p|p==&selected) { paths=siblings; self.index=index; }
            }
        }
        self.paths=paths; self.load(ctx);
    }
    fn load(&mut self,ctx:&Context) {
        let Some(path)=self.paths.get(self.index).cloned() else { return; };
        ctx.send_viewport_cmd(ViewportCommand::Title(format!("{} — wiu",path.file_name().unwrap_or_default().to_string_lossy())));
        self.picture=None; self.texture=None; self.error=None; self.zoom=None; self.offset=Vec2::ZERO; self.frame=0;self.paused=false;
        let (tx,rx)=mpsc::channel(); self.receiver=Some(rx); let ctx=ctx.clone();
        let request=REQUEST.fetch_add(1,std::sync::atomic::Ordering::SeqCst)+1;
        std::thread::spawn(move|| {
            let _guard=DECODE_LOCK.lock().unwrap_or_else(|e|e.into_inner());
            if request!=REQUEST.load(std::sync::atomic::Ordering::SeqCst){return;}
            let result=std::panic::catch_unwind(||decode::load(&path)).unwrap_or_else(|_|Err("Le décodeur n’a pas pu lire ce fichier.".into()));
            let _=tx.send(result);ctx.request_repaint();
        });
    }
    fn choose(&mut self,ctx:&Context) { match platform::choose() { Ok(p) => self.open(p,ctx),Err(e)=>self.error=Some(e) } }
    fn navigate(&mut self,delta:isize,ctx:&Context) { if !self.paths.is_empty() { self.index=(self.index as isize+delta).rem_euclid(self.paths.len() as isize) as usize;self.load(ctx); } }
    fn upload(&mut self,ctx:&Context) { if let Some(p)=&self.picture { let f=&p.frames[self.frame]; self.texture=Some(ctx.load_texture("photo",ColorImage::from_rgba_unmultiplied([p.width as usize,p.height as usize],&f.rgba),TextureOptions::LINEAR)); } }
}
impl eframe::App for Viewer {
    fn ui(&mut self,ui:&mut Ui,_:&mut eframe::Frame) {
        let ctx=ui.ctx().clone();
        if self.initial { self.initial=false; if self.paths.is_empty() {self.choose(&ctx)} else {self.open(self.paths.clone(),&ctx)} }
        let dropped:Vec<_>=ctx.input(|i|i.raw.dropped_files.iter().map(|f|f.path().to_path_buf()).collect());if !dropped.is_empty(){self.open(dropped,&ctx)}
        if ctx.input_mut(|i|i.consume_key(Modifiers::CTRL,Key::O)) { self.choose(&ctx); }
        if ctx.input(|i|i.key_pressed(Key::ArrowLeft)) {self.navigate(-1,&ctx)}
        if ctx.input(|i|i.key_pressed(Key::ArrowRight)) {self.navigate(1,&ctx)}
        if ctx.input(|i|i.key_pressed(Key::Num0)) {self.zoom=None;self.offset=Vec2::ZERO;}
        if ctx.input(|i|i.key_pressed(Key::Num1)) {self.zoom=Some(1.);self.offset=Vec2::ZERO;}
        if ctx.input(|i|i.key_pressed(Key::Space)) {self.paused=!self.paused;self.deadline=Instant::now();}
        if ctx.input(|i|i.key_pressed(Key::F11)) {self.fullscreen=!self.fullscreen;ctx.send_viewport_cmd(ViewportCommand::Fullscreen(self.fullscreen));}
        if ctx.input(|i|i.key_pressed(Key::Escape)) && self.fullscreen {self.fullscreen=false;ctx.send_viewport_cmd(ViewportCommand::Fullscreen(false));}
        if let Some(result)=self.receiver.as_ref().and_then(|rx|rx.try_recv().ok()) { self.receiver=None; match result {Ok(p)=>{self.picture=Some(p);self.upload(&ctx);self.deadline=Instant::now()+Duration::from_millis(self.picture.as_ref().unwrap().frames[0].delay as u64)},Err(e)=>self.error=Some(e)} }
        if let Some(p)=&self.picture { if p.frames.len()>1&&!self.paused { if Instant::now()>=self.deadline {self.frame=(self.frame+1)%p.frames.len();self.deadline=Instant::now()+Duration::from_millis(p.frames[self.frame].delay as u64);self.upload(&ctx);}ctx.request_repaint_after(self.deadline.saturating_duration_since(Instant::now())); } }
        egui::Panel::top("toolbar").show(ui,|ui| {ui.horizontal(|ui| {
            ui.heading(RichText::new("wiu").color(Color32::from_rgb(103,216,190)));ui.separator();
            if ui.button("Ouvrir…").clicked(){self.choose(&ctx)}
            if navigation_button(ui, false).clicked(){self.navigate(-1,&ctx)} if navigation_button(ui, true).clicked(){self.navigate(1,&ctx)}
            if ui.button("Adapter").clicked(){self.zoom=None;self.offset=Vec2::ZERO;}
            if ui.button("100 %").clicked(){self.zoom=Some(1.);self.offset=Vec2::ZERO;}
            if self.picture.as_ref().is_some_and(|p|p.frames.len()>1)&&ui.button(if self.paused {"Lire"} else {"Pause"}).clicked(){self.paused=!self.paused;}
            if let Some(label) = self.installation_action && ui.button(label).clicked() {
                match platform::install() {
                    Ok(message) => { self.installation_action = None; platform::message(&message); }
                    Err(error) => platform::message(&format!("Erreur : {error}")),
                }
            }
        });});
        let area=ui.available_rect_before_wrap();
        let response=ui.allocate_rect(area,Sense::drag());
        let painter=ui.painter_at(area);painter.rect_filled(area,0.,Color32::from_rgb(18,21,27));
        if let (Some(p),Some(t))=(&self.picture,&self.texture) {
            let fit=(area.width()/p.width as f32).min((area.height()-36.)/p.height as f32).min(1.);
            let mut zoom=self.zoom.unwrap_or(fit);
            if response.hovered() {
                let scroll = ctx.input(|i| i.smooth_scroll_delta.y);
                if scroll != 0. && let Some(cursor) = response.hover_pos() {
                    let next_zoom = (zoom * (scroll * 0.002).exp()).clamp(0.01, 64.);
                    self.offset = offset_after_zoom(self.offset, area.center(), cursor, next_zoom / zoom);
                    zoom = next_zoom;
                    self.zoom = Some(zoom);
                }
            }
            if response.dragged(){self.offset+=response.drag_delta();}
            let rect=Rect::from_center_size(area.center()+self.offset,vec2(p.width as f32*zoom,p.height as f32*zoom));
            let visible=rect.intersect(area); let step=20.;
            for y in 0..(visible.height()/step).ceil() as i32 {for x in 0..(visible.width()/step).ceil() as i32 {let r=Rect::from_min_size(visible.min+vec2(x as f32*step,y as f32*step),vec2(step,step)).intersect(visible);painter.rect_filled(r,0.,Color32::from_gray(if (x+y)%2==0 {45}else{58}));}}
            painter.image(t.id(),rect,Rect::from_min_max(Pos2::ZERO,pos2(1.,1.)),Color32::WHITE);
            let name=self.paths.get(self.index).and_then(|p|p.file_name()).unwrap_or_default().to_string_lossy();
            let text=format!("{name}  ·  {} × {}  ·  {:.0} %  ·  {}/{}",p.width,p.height,zoom*100.,self.index+1,self.paths.len());
            painter.rect_filled(Rect::from_min_max(pos2(area.left(),area.bottom()-28.),area.max),0.,Color32::from_black_alpha(210));
            painter.text(pos2(area.left()+12.,area.bottom()-14.),Align2::LEFT_CENTER,text,FontId::proportional(13.),Color32::LIGHT_GRAY);
        } else { let text=if let Some(e)=&self.error {e.as_str()}else if self.receiver.is_some(){"Chargement…"}else{"Ouvrez une image (Ctrl+O) ou déposez-la ici"};painter.text(area.center(),Align2::CENTER_CENTER,text,FontId::proportional(17.),Color32::LIGHT_GRAY); }
    }
}

#[cfg(test)]
mod zoom_tests {
    use super::*;

    #[test]
    fn cursor_keeps_same_image_point_after_zoom() {
        let center = pos2(540., 380.);
        for offset in [Vec2::ZERO, vec2(135., -72.)] {
            for cursor in [center, pos2(120., 210.), pos2(960., 670.)] {
                for (before, after) in [(0.5, 2.), (2., 0.5), (32., 64.), (0.02, 0.01), (64., 64.)] {
                    let point = (cursor - (center + offset)) / before;
                    let new_offset = offset_after_zoom(offset, center, cursor, after / before);
                    let position = center + new_offset + point * after;
                    assert!((position - cursor).length() < 0.001);
                }
            }
        }
    }
}
