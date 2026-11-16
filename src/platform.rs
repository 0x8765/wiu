use std::{path::PathBuf,process::Command,os::windows::process::CommandExt};
use windows::{core::{HSTRING,w},Win32::{System::Com::*, UI::Shell::*}};
pub struct Com;
impl Com { pub fn new()->windows::core::Result<Self>{unsafe{CoInitializeEx(None,COINIT_APARTMENTTHREADED).ok()?;}Ok(Self)} }
impl Drop for Com {fn drop(&mut self){unsafe{CoUninitialize()}}}
pub fn message(s:&str) { use windows::Win32::UI::WindowsAndMessaging::*; unsafe {MessageBoxW(None,&HSTRING::from(s),w!("wiu"),MB_OK);}}
pub fn choose()->Result<Vec<PathBuf>,String>{ (||->windows::core::Result<_>{
    let _com=Com::new()?;
    unsafe {
        let dialog:IFileOpenDialog=CoCreateInstance(&FileOpenDialog,None,CLSCTX_INPROC_SERVER)?;
        dialog.SetOptions(FOS_ALLOWMULTISELECT|FOS_FILEMUSTEXIST|FOS_PATHMUSTEXIST)?;
        dialog.SetTitle(w!("wiu — Choisir des images"))?;
        dialog.SetFileTypes(&[Common::COMDLG_FILTERSPEC{pszName:w!("Images"),pszSpec:w!("*.jpg;*.jpeg;*.jfif;*.png;*.gif;*.webp;*.bmp;*.tif;*.tiff;*.ico;*.avif;*.svg;*.pnm;*.ppm;*.pgm;*.pbm;*.tga")},Common::COMDLG_FILTERSPEC{pszName:w!("Tous les fichiers"),pszSpec:w!("*.*")}])?;
        if let Err(e)=dialog.Show(None){if e.code().0 as u32==0x800704c7 {return Ok(Vec::new());}return Err(e);}
        let items=dialog.GetResults()?;let mut paths=Vec::new();
        for i in 0..items.GetCount()? {let p=items.GetItemAt(i)?.GetDisplayName(SIGDN_FILESYSPATH)?;let text=p.to_string()?;CoTaskMemFree(Some(p.0.cast()));paths.push(PathBuf::from(text));}
        Ok(paths)
    }
})().map_err(|e|e.to_string()) }
fn reg(args:&[&str])->Result<(),String>{let out=Command::new("reg.exe").args(args).creation_flags(0x08000000).output().map_err(|e|e.to_string())?;if out.status.success(){Ok(())}else{Err(String::from_utf8_lossy(&out.stderr).into_owned())}}
fn set(key:&str,name:&str,value:&str)->Result<(),String>{let mut a=vec!["add",key];if name.is_empty(){a.push("/ve")}else{a.extend(["/v",name])}a.extend(["/t","REG_SZ","/d",value,"/f"]);reg(&a)}
fn registry_string(key: &str, name: &str) -> Option<String> {
    use windows::Win32::System::Registry::*;
    unsafe {
        let key = HSTRING::from(key);
        let name = HSTRING::from(name);
        let mut bytes = 0;
        if RegGetValueW(HKEY_CURRENT_USER, &key, &name, RRF_RT_REG_SZ, None, None, Some(&mut bytes)).is_err() {
            return None;
        }
        let mut value = vec![0u16; (bytes as usize + 1) / 2];
        if RegGetValueW(HKEY_CURRENT_USER, &key, &name, RRF_RT_REG_SZ, None, Some(value.as_mut_ptr().cast()), Some(&mut bytes)).is_err() {
            return None;
        }
        let end = value.iter().position(|&c| c == 0).unwrap_or(value.len());
        String::from_utf16(&value[..end]).ok()
    }
}
pub fn is_installed() -> bool {
    let Some(local) = std::env::var_os("LOCALAPPDATA") else { return false; };
    let exe = PathBuf::from(local).join("Programs\\wiu\\wiu.exe");
    let command = format!("\"{}\" \"%1\"", exe.display());
    exe.is_file()
        && registry_string("Software\\RegisteredApplications", "wiu").as_deref() == Some("Software\\wiu\\Capabilities")
        && registry_string("Software\\Classes\\Applications\\wiu.exe\\shell\\open\\command", "").as_deref() == Some(command.as_str())
}
fn same_file_contents(left: &std::path::Path, right: &std::path::Path) -> std::io::Result<bool> {
    use std::io::Read;
    let mut left = std::fs::File::open(left)?;
    let mut right = std::fs::File::open(right)?;
    if left.metadata()?.len() != right.metadata()?.len() { return Ok(false); }
    let mut a = [0u8; 65536];
    let mut b = [0u8; 65536];
    loop {
        let n = left.read(&mut a)?;
        if n == 0 { return Ok(true); }
        right.read_exact(&mut b[..n])?;
        if a[..n] != b[..n] { return Ok(false); }
    }
}
pub fn installation_action() -> Option<&'static str> {
    if !is_installed() { return Some("Installer"); }
    let source = std::env::current_exe().ok()?;
    let installed = PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join("Programs\\wiu\\wiu.exe");
    if same_file_contents(&source, &installed).unwrap_or(false) { None } else { Some("Mettre à jour") }
}
pub fn install()->Result<String,String>{
    let source=std::env::current_exe().map_err(|e|e.to_string())?;
    let dir=PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA absent")?).join("Programs\\wiu");
    std::fs::create_dir_all(&dir).map_err(|e|e.to_string())?;let exe=dir.join("wiu.exe");
    if source!=exe {std::fs::copy(&source,&exe).map_err(|e|format!("Impossible de remplacer la version installée : {e}\nFermez les autres fenêtres de wiu, puis réessayez depuis ce nouvel exécutable."))?;}
    let path=exe.to_string_lossy();let command=format!("\"{path}\" \"%1\"");
    let base="HKCU\\Software\\Classes";
    set(&format!("{base}\\Applications\\wiu.exe"),"FriendlyAppName","wiu")?;
    set(&format!("{base}\\Applications\\wiu.exe\\shell\\open\\command"),"",&command)?;
    set(&format!("{base}\\wiu.Image"),"","Image wiu")?;
    set(&format!("{base}\\wiu.Image\\shell\\open\\command"),"",&command)?;
    set(&format!("{base}\\wiu.Image\\DefaultIcon"),"",&format!("\"{path}\",0"))?;
    set("HKCU\\Software\\wiu\\Capabilities","ApplicationName","wiu")?;
    set("HKCU\\Software\\wiu\\Capabilities","ApplicationDescription","Visionneuse d’images")?;
    set("HKCU\\Software\\RegisteredApplications","wiu","Software\\wiu\\Capabilities")?;
    for ext in crate::EXTENSIONS {
        set(&format!("{base}\\Applications\\wiu.exe\\SupportedTypes"),&format!(".{ext}"),"")?;
        set(&format!("HKCU\\Software\\wiu\\Capabilities\\FileAssociations"),&format!(".{ext}"),"wiu.Image")?;
        let key=format!("{base}\\SystemFileAssociations\\.{ext}\\shell\\wiu");
        set(&key,"","Ouvrir avec wiu")?;set(&key,"Icon",&format!("\"{path}\",0"))?;set(&format!("{key}\\command"),"",&command)?;
    }
    unsafe{SHChangeNotify(SHCNE_ASSOCCHANGED,SHCNF_IDLIST,None,None);}
    Ok(format!("wiu est disponible dans « Ouvrir avec » et le menu contextuel.\nInstallé dans {}",exe.display()))
}
pub fn uninstall()->Result<String,String>{
    let base="HKCU\\Software\\Classes";let mut errors=Vec::new();
    let mut keys=vec![format!("{base}\\Applications\\wiu.exe"),format!("{base}\\wiu.Image"),"HKCU\\Software\\wiu".into()];
    keys.extend(crate::EXTENSIONS.iter().map(|e|format!("{base}\\SystemFileAssociations\\.{e}\\shell\\wiu")));
    for key in keys {if reg(&["query",&key]).is_ok(){if let Err(e)=reg(&["delete",&key,"/f"]){errors.push(e)}}}
    if reg(&["query","HKCU\\Software\\RegisteredApplications","/v","wiu"]).is_ok(){if let Err(e)=reg(&["delete","HKCU\\Software\\RegisteredApplications","/v","wiu","/f"]){errors.push(e)}}
    unsafe{SHChangeNotify(SHCNE_ASSOCCHANGED,SHCNF_IDLIST,None,None);}
    if !errors.is_empty(){return Err(errors.join("\n"));}
    Ok("Intégration retirée. Vous pouvez fermer wiu puis supprimer %LOCALAPPDATA%\\Programs\\wiu.".into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn detect_replacement_with_same_size() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/fixtures/update");
        std::fs::create_dir_all(&dir).unwrap();
        let portable = dir.join("portable.exe");
        let installed = dir.join("installed.exe");
        let mut bytes = vec![42u8; 100_000];
        std::fs::write(&portable, &bytes).unwrap();
        std::fs::write(&installed, &bytes).unwrap();
        assert!(super::same_file_contents(&portable, &installed).unwrap());
        bytes[99_999] = 43;
        std::fs::write(&portable, &bytes).unwrap();
        assert!(!super::same_file_contents(&portable, &installed).unwrap());
    }
}
