use std::{path::Path,ptr};
use windows::{core::HSTRING,Win32::{Foundation::GENERIC_READ,Graphics::Imaging::*,System::Com::*,System::Com::StructuredStorage::*}};
pub struct Frame {pub rgba:Vec<u8>,pub delay:u32}
pub struct Picture {pub width:u32,pub height:u32,pub frames:Vec<Frame>}
const LIMIT:usize=256*1024*1024;
pub fn size(w:u32,h:u32)->Result<usize,String>{let n=(w as u64).checked_mul(h as u64).and_then(|n|n.checked_mul(4)).unwrap_or(u64::MAX);if w==0||h==0||n>LIMIT as u64{Err("Image trop volumineuse (limite : 256 Mio décodés).".into())}else{Ok(n as usize)}}
pub fn load(path:&Path)->Result<Picture,String>{
    let meta=std::fs::metadata(path).map_err(|e|format!("Impossible d’ouvrir {} : {e}",path.display()))?;
    if meta.len()>128*1024*1024{return Err("Fichier trop volumineux (limite : 128 Mio).".into());}
    let ext=path.extension().unwrap_or_default().to_string_lossy().to_ascii_lowercase();
    if ext=="svg" {return crate::svg::load(path);}
    if ["gif","webp","avif"].contains(&ext.as_str()){return wic(path).map_err(|e|format!("Décodage {ext} impossible : {e}. Pour WebP/AVIF, vérifier les extensions d’images Windows."));}
    let mut reader=image::ImageReader::open(path).map_err(|e|e.to_string())?.with_guessed_format().map_err(|e|e.to_string())?;
    let mut limits=image::Limits::default();limits.max_alloc=Some(LIMIT as u64);limits.max_image_width=Some(32768);limits.max_image_height=Some(32768);reader.limits(limits);
    use image::ImageDecoder;
    let mut decoder=reader.into_decoder().map_err(|e|e.to_string())?;let orientation=decoder.orientation().unwrap_or(image::metadata::Orientation::NoTransforms);
    let (w,h)=decoder.dimensions();size(w,h)?;
    let mut image=image::DynamicImage::from_decoder(decoder).map_err(|e|e.to_string())?;image.apply_orientation(orientation);
    Ok(Picture{width:image.width(),height:image.height(),frames:vec![Frame{rgba:image.to_rgba8().into_raw(),delay:100}]})
}
unsafe fn metadata(reader:Option<&IWICMetadataQueryReader>,name:&str,default:u32)->u32{unsafe{
    let Some(r)=reader else{return default};let mut v=PROPVARIANT::default();
    if r.GetMetadataByName(&HSTRING::from(name),&mut v).is_err(){return default;}
    let value=PropVariantToUInt32(&v).unwrap_or(default);let _=PropVariantClear(&mut v);value
}}
fn wic(path:&Path)->Result<Picture,String>{(unsafe{(||->Result<Picture,Box<dyn std::error::Error>>{
    let _com=crate::platform::Com::new()?;
    let factory:IWICImagingFactory=CoCreateInstance(&CLSID_WICImagingFactory,None,CLSCTX_INPROC_SERVER)?;
    let decoder=factory.CreateDecoderFromFilename(&HSTRING::from(path.as_os_str()),None,GENERIC_READ,WICDecodeMetadataCacheOnDemand)?;
    let gif=path.extension().is_some_and(|s|s.eq_ignore_ascii_case("gif"));
    let first=decoder.GetFrame(0)?;let(mut w,mut h)=(0,0);first.GetSize(&mut w,&mut h)?;
    if gif {let r=decoder.GetMetadataQueryReader().ok();w=metadata(r.as_ref(),"/logscrdesc/Width",w);h=metadata(r.as_ref(),"/logscrdesc/Height",h);}
    let bytes=size(w,h)?;let count=if gif {decoder.GetFrameCount()?}else{1};
    if count as u64*bytes as u64>LIMIT as u64{return Err("Animation trop volumineuse (256 Mio).".into());}
    let mut canvas=vec![0;bytes];let mut frames=Vec::new();
    for i in 0..count {
        let frame=decoder.GetFrame(i)?;let(mut fw,mut fh)=(0,0);frame.GetSize(&mut fw,&mut fh)?;let n=size(fw,fh)?;
        let converter=factory.CreateFormatConverter()?;converter.Initialize(&frame,&GUID_WICPixelFormat32bppRGBA,WICBitmapDitherTypeNone,None,0.,WICBitmapPaletteTypeCustom)?;
        let mut pixels=vec![0;n];converter.CopyPixels(ptr::null(),fw*4,&mut pixels)?;
        let reader=frame.GetMetadataQueryReader().ok();let left=metadata(reader.as_ref(),"/imgdesc/Left",0);let top=metadata(reader.as_ref(),"/imgdesc/Top",0);
        let disposal=metadata(reader.as_ref(),"/grctlext/Disposal",0);let delay=metadata(reader.as_ref(),"/grctlext/Delay",10).saturating_mul(10).max(20);
        let previous=if disposal==3{Some(canvas.clone())}else{None};
        for y in 0..fh.min(h.saturating_sub(top)) {for x in 0..fw.min(w.saturating_sub(left)){let src=((y*fw+x)*4)as usize;let dst=(((y+top)*w+x+left)*4)as usize;if pixels[src+3]!=0{canvas[dst..dst+4].copy_from_slice(&pixels[src..src+4]);}}}
        frames.push(Frame{rgba:canvas.clone(),delay});
        if let Some(previous)=previous{canvas=previous}else if disposal==2 {for y in top..top.saturating_add(fh).min(h){for x in left..left.saturating_add(fw).min(w){let d=((y*w+x)*4)as usize;canvas[d..d+4].fill(0);}}}
    }
    Ok(Picture{width:w,height:h,frames})
})()}).map_err(|e|e.to_string())}

#[cfg(test)]mod tests {
    use super::*;
    fn fixtures()->std::path::PathBuf {let p=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/fixtures");std::fs::create_dir_all(&p).unwrap();p}
    #[test]fn allocation_bounds(){assert!(size(0,10).is_err());assert!(size(u32::MAX,u32::MAX).is_err());assert_eq!(size(20,10).unwrap(),800);}
    #[test]fn bitmap_roundtrips(){let dir=fixtures();let img=image::RgbaImage::from_pixel(12,8,image::Rgba([220,40,80,255]));for ext in ["png","jpg","bmp","tiff","ico","pnm","tga"] {let path=dir.join(format!("été image.{ext}"));if ext=="ico" { img.save(&path).unwrap(); } else { image::DynamicImage::ImageRgba8(img.clone()).to_rgb8().save(&path).unwrap(); }let p=load(&path).unwrap();assert_eq!((p.width,p.height),(12,8));assert_eq!(p.frames[0].rgba.len(),384);}}
    #[test]fn gif_and_bad_files(){let dir=fixtures();let path=dir.join("tiny.gif");std::fs::write(&path,b"GIF89a\x01\x00\x01\x00\x80\x00\x00\x00\x00\x00\xff\xff\xff\x21\xf9\x04\x01\x00\x00\x00\x00\x2c\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x02\x44\x01\x00\x3b").unwrap();let p=load(&path).unwrap();assert_eq!((p.width,p.height),(1,1));let bad=dir.join("bad.png");std::fs::write(&bad,b"not an image").unwrap();assert!(load(&bad).is_err());assert!(load(&dir.join("absent.jpg")).is_err());}
    #[test]fn svg_render(){let dir=fixtures();let path=dir.join("test.svg");std::fs::write(&path,r##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="24"><rect width="32" height="24" fill="#ff0000"/></svg>"##).unwrap();let p=load(&path).unwrap();assert_eq!((p.width,p.height),(32,24));assert_eq!(&p.frames[0].rgba[..4],&[255,0,0,255]);let remote=dir.join("remote.svg");std::fs::write(&remote,r#"<svg xmlns="http://www.w3.org/2000/svg"><image href="https://example.org/a.png"/></svg>"#).unwrap();assert!(load(&remote).is_err());}
    #[test]fn animated_gif(){
        let path=fixtures().join("animated.gif");
        let mut data=b"GIF89a\x01\x00\x01\x00\x80\x00\x00\xff\x00\x00\x00\x00\xff".to_vec();
        for code in [0x44,0x4c] {
            data.extend_from_slice(b"\x21\xf9\x04\x00\x05\x00\x00\x00\x2c\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x02");
            data.extend_from_slice(&[code,1,0]);
        }
        data.push(0x3b);std::fs::write(&path,data).unwrap();
        let p=load(&path).unwrap();assert_eq!(p.frames.len(),2);
        assert_eq!(p.frames[0].delay,50);
        assert_eq!(&p.frames[0].rgba[..4],&[255,0,0,255]);
        assert_eq!(&p.frames[1].rgba[..4],&[0,0,255,255]);
    }
    #[test]fn jpeg_orientation(){
        let path=fixtures().join("orientation.jpg");
        image::RgbImage::new(12,8).save(&path).unwrap();
        let jpg=std::fs::read(&path).unwrap();
        let exif=b"Exif\x00\x00II\x2a\x00\x08\x00\x00\x00\x01\x00\x12\x01\x03\x00\x01\x00\x00\x00\x06\x00\x00\x00\x00\x00\x00\x00";
        let mut data=vec![255,216,255,225];data.extend_from_slice(&((exif.len()+2)as u16).to_be_bytes());data.extend_from_slice(exif);data.extend_from_slice(&jpg[2..]);
        std::fs::write(&path,&data).unwrap();let p=load(&path).unwrap();assert_eq!((p.width,p.height),(8,12));assert_eq!(std::fs::read(&path).unwrap(),data);
    }

}
