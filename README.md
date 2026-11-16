# wiu

Visionneuse d’images Windows x64, écrite en Rust. Aucune édition ni modification des photos.

## Utilisation

Ouvrir `dist/wiu.exe` : l’application démarre dans une fenêtre maximisée, avec la barre de titre Windows, et un dialogue permet de choisir une ou plusieurs images. Annuler laisse la fenêtre prête à ouvrir une image. Le glisser-déposer est également accepté. F11 active le plein écran ; F11 ou Échap permettent d’en sortir.

```powershell
.\dist\wiu.exe "C:\Photos\été 2026.jpg"
.\dist\wiu.exe "C:\Photos\a.png" "C:\Photos\b.tiff"
```

| Commande | Action |
| --- | --- |
| Ctrl+O | Choisir des images |
| Flèches gauche/droite | Image précédente/suivante |
| Molette | Zoom autour du curseur |
| Glisser avec le bouton gauche | Déplacer l’image |
| 0 / Adapter | Adapter à la fenêtre |
| 1 / 100 % | Taille réelle |
| F11 / Échap | Plein écran / quitter le plein écran |
| Espace | Pause/reprise du GIF |

Une image seule permet de parcourir les images compatibles du même dossier, triées par nom. Plusieurs chemins permettent de parcourir uniquement la sélection. L’affichage respecte l’orientation EXIF des formats décodés par `image`. L’ouverture des fichiers ne modifie jamais leurs octets.

## Explorateur Windows

Dans la fenêtre, cliquer sur **Installer**, ou lancer :

```powershell
.\dist\wiu.exe --install
```

Cela copie wiu dans `%LOCALAPPDATA%\Programs\wiu\wiu.exe` et ajoute, pour le compte courant, l’application à **Ouvrir avec** et l’action **Ouvrir avec wiu**. Le bouton **Installer** disparaît après une installation réussie. Si vous lancez un autre exécutable dont le contenu diffère de la copie installée, un bouton **Mettre à jour** apparaît. Il disparaît après la mise à jour ; lorsque les deux copies sont identiques, aucun bouton d’installation n’apparaît. Fermez les autres fenêtres de wiu avant de mettre à jour. Sous Windows 11, l’action classique peut se trouver dans **Afficher plus d’options**. Aucun droit administrateur n’est nécessaire. L’application par défaut au double-clic reste au choix de l’utilisateur dans Windows.

Pour retirer ces entrées : `wiu.exe --uninstall`. Fermer ensuite wiu et supprimer son dossier installé si souhaité. L’installation est répétable ; pour remplacer un exécutable installé, fermer d’abord cette copie et lancer l’installation depuis la nouvelle version portable.

## Formats et limites

- JPEG/JPG/JFIF, PNG, BMP, TIFF, ICO, PNM/PPM/PGM/PBM et TGA : décodeurs Rust intégrés.
- GIF : décodeur Windows intégré, animations avec transparence, délais et restauration des images.
- WebP et AVIF : première image, via les codecs WIC disponibles sur Windows. Les extensions d’images WebP/AV1 de Windows peuvent être nécessaires. Le logiciel affiche une erreur explicite si le codec est absent. Ces deux formats ne sont donc pas garantis sur une installation Windows sans codecs.
- SVG : rendu natif Direct2D de Windows 10/11, image statique aux dimensions du document. Certaines fonctions SVG complexes (filtres, scripts, animations) ne sont pas prises en charge par Direct2D. Les références externes sont refusées.
- TIFF multipage : première page ; ICO : représentation choisie par le décodeur. Pas de HEIC ni de RAW, ni de gestion avancée des profils couleur/HDR.
- Fichier limité à 128 Mio ; pixels décodés ou ensemble des images d’un GIF limités à 256 Mio. Les chargements sont sérialisés hors de l’interface ; les demandes périmées sont ignorées.

## Compilation

Rust stable (au moins 1.92 pour l’interface) avec la cible `x86_64-pc-windows-gnu` et MinGW (`gcc`, `windres`) dans le PATH.

```powershell
cargo build --release --locked
cargo test --locked
```

Les versions sont verrouillées dans `Cargo.lock`. Ici, la compilation utilise les paquets déjà en cache (`--offline`), car crates.io est inaccessible. L’interface utilise eframe/egui ; les dialogues, GIF et codecs optionnels utilisent Windows ; SVG utilise Direct2D en rendu logiciel. Aucun téléchargement n’est effectué à l’exécution.

Le binaire est produit dans `target/release/wiu.exe`. Exécuter `python tools/package.py` après compilation pour actualiser `dist`, la notice et les licences des dépendances.

Vérification du décodage sans fenêtre : `wiu.exe --check "image.png" "image.svg"` (code de sortie 0 si tous les fichiers sont lisibles, 1 sinon). Les tests créent leurs échantillons dans `target/fixtures`.

Les modifications au registre sont limitées aux clés `HKCU\Software\Classes\Applications\wiu.exe`, `HKCU\Software\Classes\wiu.Image`, aux verbes `wiu` sous `SystemFileAssociations`, à `HKCU\Software\wiu` et à la valeur `wiu` de `RegisteredApplications`.
