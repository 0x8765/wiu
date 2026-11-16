use std::path::Path;
use crate::decode::{Picture,Frame};
use windows::{core::Interface,Win32::{Foundation::HMODULE,Graphics::{Direct2D::*,Direct2D::Common::*,Direct3D::*,Direct3D11::*,Dxgi::*,Dxgi::Common::*},UI::Shell::SHCreateMemStream}};
pub fn load(path:&Path)->Result<Picture,String>{
    let xml=std::fs::read_to_string(path).map_err(|e|e.to_string())?;
    let doc=roxmltree::Document::parse(&xml).map_err(|e|e.to_string())?;
    let root=doc.root_element();if root.tag_name().name()!="svg"{return Err("Document SVG attendu.".into());}
    for node in doc.descendants(){for a in node.attributes(){if a.name()=="href"&&!a.value().starts_with('#'){return Err("SVG : les ressources externes ne sont pas chargées.".into());}}}
    let view:Vec<f32>=root.attribute("viewBox").unwrap_or("").split(|c:char|c==','||c.is_whitespace()).filter_map(|s|s.parse().ok()).collect();
    let dim=|name:&str,index:usize,default:f32|root.attribute(name).and_then(|s|s.trim_end_matches("px").parse::<f32>().ok()).unwrap_or_else(||view.get(index).copied().unwrap_or(default));
    let w=dim("width",2,800.).ceil() as u32;let h=dim("height",3,600.).ceil() as u32;let n=crate::decode::size(w,h)?;
    (unsafe{(||->windows::core::Result<Picture>{
        let _com=crate::platform::Com::new()?;
        let mut device=None;
        D3D11CreateDevice(None,D3D_DRIVER_TYPE_WARP,HMODULE::default(),D3D11_CREATE_DEVICE_BGRA_SUPPORT,None,D3D11_SDK_VERSION,Some(&mut device),None,None)?;
        let dxgi:IDXGIDevice=device.unwrap().cast()?;
        let device=D2D1CreateDevice(&dxgi,None)?;
        let ctx:ID2D1DeviceContext5=device.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?.cast()?;
        let stream=SHCreateMemStream(Some(xml.as_bytes())).ok_or_else(windows::core::Error::from_thread)?;
        let svg=ctx.CreateSvgDocument(&stream,D2D_SIZE_F{width:w as f32,height:h as f32})?;
        let mut props=D2D1_BITMAP_PROPERTIES1{pixelFormat:D2D1_PIXEL_FORMAT{format:DXGI_FORMAT_B8G8R8A8_UNORM,alphaMode:D2D1_ALPHA_MODE_PREMULTIPLIED},dpiX:96.,dpiY:96.,bitmapOptions:D2D1_BITMAP_OPTIONS_TARGET,..Default::default()};
        let target=ctx.CreateBitmap(D2D_SIZE_U{width:w,height:h},None,0,&props)?;
        ctx.SetTarget(&target);ctx.BeginDraw();ctx.Clear(Some(&D2D1_COLOR_F{r:0.,g:0.,b:0.,a:0.}));ctx.DrawSvgDocument(&svg);ctx.EndDraw(None,None)?;
        ctx.SetTarget(None);
        props.bitmapOptions=D2D1_BITMAP_OPTIONS_CPU_READ|D2D1_BITMAP_OPTIONS_CANNOT_DRAW;
        let cpu=ctx.CreateBitmap(D2D_SIZE_U{width:w,height:h},None,0,&props)?;cpu.CopyFromBitmap(None,&target,None)?;
        let mapped=cpu.Map(D2D1_MAP_OPTIONS_READ)?;let mut rgba=vec![0;n];
        for y in 0..h as usize {let row=std::slice::from_raw_parts(mapped.bits.add(y*mapped.pitch as usize),w as usize*4);for x in 0..w as usize {let s=&row[x*4..x*4+4];let d=&mut rgba[(y*w as usize+x)*4..][..4];let alpha=s[3] as u32;d[3]=s[3];for c in 0..3{d[c]=if alpha==0{0}else{((s[2-c] as u32*255+alpha/2)/alpha).min(255) as u8};}}}
        cpu.Unmap()?;Ok(Picture{width:w,height:h,frames:vec![Frame{rgba,delay:100}]})
    })()}).map_err(|e|format!("SVG non pris en charge par Direct2D : {e}"))
}
