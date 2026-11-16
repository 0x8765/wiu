# wiu

[Français](README.md) · [English](README.en.md) · [简体中文](README.zh-CN.md) · **Español** · [Русский](README.ru.md)

Visor de imágenes para Windows x64, escrito en Rust. Permite ver fotos sin editarlas ni modificarlas.

## Uso

Si utilizas el código fuente, primero compila la aplicación y genera la carpeta `dist/` siguiendo la sección [Compilación](#compilación). Las carpetas `target/` y `dist/` se generan localmente y no se incluyen en el control de versiones de Git.

Abre `dist/wiu.exe`. La aplicación se inicia en una ventana maximizada, con la barra de título de Windows y un diálogo para seleccionar una o varias imágenes. Si cancelas, la ventana queda lista para abrir una imagen. También puedes arrastrar y soltar imágenes. F11 activa la pantalla completa; F11 o Esc permiten salir de ella.

```powershell
.\dist\wiu.exe "C:\Photos\summer 2026.jpg"
.\dist\wiu.exe "C:\Photos\a.png" "C:\Photos\b.tiff"
```

| Atajo | Acción |
| --- | --- |
| Ctrl+O | Elegir imágenes |
| Flechas izquierda/derecha | Imagen anterior/siguiente |
| Rueda del ratón | Ampliar o reducir alrededor del cursor |
| Arrastrar con el botón izquierdo | Desplazar la imagen |
| 0 / Adapter | Ajustar a la ventana |
| 1 / 100 % | Tamaño real |
| F11 / Esc | Activar/salir de pantalla completa |
| Espacio | Pausar/reanudar el GIF |

Al abrir una sola imagen, puedes recorrer las imágenes compatibles de la misma carpeta, ordenadas por nombre. Si proporcionas varias rutas, solo se recorre esa selección. Se respeta la orientación EXIF de los formatos decodificados por `image`. Abrir un archivo nunca modifica sus bytes.

## Integración con el Explorador de Windows

Haz clic en **Installer** en la ventana de la aplicación, o ejecuta:

```powershell
.\dist\wiu.exe --install
```

Esto copia wiu en `%LOCALAPPDATA%\Programs\wiu\wiu.exe` y registra la aplicación en **Abrir con**, junto con la acción **Ouvrir avec wiu** del menú contextual, para el usuario actual. El botón **Installer** desaparece tras una instalación correcta. Si abres otro ejecutable cuyo contenido difiere de la copia instalada, aparece el botón **Mettre à jour**. Desaparece después de actualizar; cuando ambas copias son idénticas, no aparece ningún botón de instalación. Cierra las demás ventanas de wiu antes de actualizar. En Windows 11, la acción clásica puede encontrarse en **Mostrar más opciones**. No se necesitan permisos de administrador. El usuario sigue eligiendo en Windows la aplicación predeterminada para abrir imágenes con doble clic.

Para eliminar estas entradas, ejecuta `wiu.exe --uninstall`. Después cierra wiu y elimina su carpeta de instalación si lo deseas. Puedes repetir la instalación; para reemplazar un ejecutable instalado, cierra primero esa copia e instala desde la nueva versión portátil.

## Formatos y límites

- JPEG/JPG/JFIF, PNG, BMP, TIFF, ICO, PNM/PPM/PGM/PBM y TGA: decodificadores Rust integrados.
- GIF: decodificador integrado de Windows, con transparencia, tiempos de los fotogramas y gestión de su restauración en las animaciones.
- WebP y AVIF: solo el primer fotograma, mediante los códecs WIC disponibles en Windows. Pueden ser necesarias las extensiones de imagen WebP/AV1 de Windows. La aplicación muestra un error claro si falta un códec. Por tanto, no se garantiza la compatibilidad con estos dos formatos en instalaciones de Windows sin los códecs correspondientes.
- SVG: renderizado nativo con Direct2D de Windows 10/11, como imagen estática con las dimensiones del documento. Direct2D no admite algunas funciones SVG complejas (filtros, scripts, animaciones). Se rechazan las referencias externas.
- TIFF multipágina: solo la primera página. ICO: representación elegida por el decodificador. Sin compatibilidad con HEIC o RAW, ni gestión avanzada de perfiles de color/HDR.
- Tamaño máximo del archivo: 128 MiB. Los píxeles decodificados, o el conjunto de fotogramas de un GIF, se limitan a 256 MiB. Las cargas se ejecutan secuencialmente fuera de la interfaz; se ignoran las solicitudes obsoletas.

## Compilación

Se necesita Rust estable (al menos 1.92 para la interfaz), con el destino `x86_64-pc-windows-gnu` y MinGW (`gcc`, `windres`) en el PATH.

```powershell
cargo build --release --locked
cargo test --locked
```

Las versiones de las dependencias están fijadas en `Cargo.lock`. Puedes añadir `--offline` a los comandos de Cargo si todas las dependencias ya están en caché. La interfaz usa eframe/egui; los diálogos, los GIF y los códecs opcionales utilizan Windows; los SVG usan el renderizado por software de Direct2D. No se descarga nada durante la ejecución.

El ejecutable se genera en `target/release/wiu.exe`. Tras compilar, genera la carpeta de distribución con Python:

```powershell
python tools/package.py
```

El script copia el ejecutable, las cinco versiones de esta documentación, la licencia del proyecto y las licencias de las dependencias en `dist/`. Utiliza `cargo metadata --offline`, por lo que las dependencias deben estar ya en caché.

Para comprobar la decodificación sin abrir una ventana, ejecuta `wiu.exe --check "image.png" "image.svg"` (código de salida 0 si todos los archivos se pueden leer, 1 en caso contrario). Las pruebas crean sus muestras en `target/fixtures`.

Los cambios en el registro se limitan a `HKCU\Software\Classes\Applications\wiu.exe`, `HKCU\Software\Classes\wiu.Image`, los verbos `wiu` en `SystemFileAssociations`, `HKCU\Software\wiu` y el valor `wiu` de `RegisteredApplications`.

## Licencia

wiu se distribuye bajo la [licencia MIT](LICENSE). Las licencias de las dependencias se incluyen en la carpeta de distribución generada.
