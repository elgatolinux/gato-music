# 🎵 Gato Music Downloader (`gato-music`)

Una herramienta de CLI ultrarrápida escrita en **Rust** para buscar y descargar música de YouTube y YouTube Music. Cuenta con soporte para múltiples formatos, descarga automática de portadas de álbumes, organización inteligente de archivos, y búsqueda/incrustación automática de letras usando la API de **LRCLIB**.

Lo mejor de todo: **No requiere que instales `yt-dlp` manualmente**. El programa lo descarga y actualiza de forma automática e invisible.

---

## 🚀 Requisitos Previos

Dado que `yt-dlp` necesita herramientas del sistema para codificar y convertir audio, asegúrate de tener instalado en tu sistema:
- **FFmpeg**: Necesario para realizar la extracción de audio a FLAC, MP3, etc.
  - En Ubuntu/Debian: `sudo apt install ffmpeg`
  - En Fedora/Arch: `sudo dnf install ffmpeg` o `sudo pacman -S ffmpeg`

---

## 🛠️ Compilación

Para compilar el proyecto desde el código fuente necesitas tener instalado Rust y Cargo ([Instalar Rust](https://www.rust-lang.org/es/tools/install)).

1. Entra al directorio del proyecto:
   ```bash
   cd gato-music
   ```

2. Compila la versión de producción (optimizada):
   ```bash
   cargo build --release
   ```

3. El binario ejecutable se generará en:
   ```bash
   ./target/release/gato-music
   ```

*(Opcional)* Si quieres instalarlo globalmente en tu sistema:
```bash
cargo install --path .
```

---

## 📖 Guía de Uso y Ejemplos

### 1. Descargar una canción por búsqueda de texto
Busca el video más relevante y lo descarga en formato FLAC en el directorio actual.
```bash
./target/release/gato-music "daft punk one more time"
```

### 2. Descargar usando un enlace directo (YouTube / YouTube Music)
```bash
./target/release/gato-music "https://www.youtube.com/watch?v=FGBhQbmPwH8"
```

### 3. Descargar un Álbum Completo (Búsqueda automática)
Busca un álbum completo en YouTube Music, crea una carpeta con el nombre del álbum y mete todas sus canciones allí (con metadatos, portadas y letras incrustadas por cada pista).
```bash
./target/release/gato-music -a "Daft Punk Discovery"
```

### 4. Cambiar el formato de salida
Puedes elegir entre `flac` (por defecto), `mp3`, `m4a` y `wav`.
```bash
./target/release/gato-music "daft punk one more time" -f mp3
```

---

## 🎛️ Flags y Opciones del CLI

Puedes consultar la ayuda del comando en cualquier momento usando la bandera `-h` o `--help`:
```bash
./target/release/gato-music --help
```

### Opciones disponibles:

| Flag / Argumento | Descripción |
|------------------|-------------|
| `<QUERY>` | **(Requerido)** El término de búsqueda de texto o la URL (video o playlist) que deseas descargar. |
| `-f, --format <FORMAT>` | El formato de audio de salida. Opciones soportadas: `flac` *(por defecto)*, `mp3`, `m4a`, `wav`. |
| `-a, --album` | Bandera para indicar que el término en `<QUERY>` debe buscarse como un álbum en YouTube Music. Descargará la playlist del álbum completo en un directorio dedicado. |
| `-h, --help` | Muestra el menú de ayuda con la descripción de todas las opciones. |
| `-V, --version` | Muestra la versión actual del programa. |

---

## ⚙️ ¿Cómo funciona internamente?

1. **Gestión de yt-dlp**: Al arrancar, busca `yt-dlp` en `~/.local/share/gato-music/yt-dlp`. Si no lo encuentra, lo descarga desde su repositorio oficial en GitHub. Si ya existe, ejecuta un comando silencioso para verificar si hay actualizaciones disponibles.
2. **Descarga y Conversión**: Invoca a `yt-dlp` para bajar el mejor audio disponible, convertirlo a tu formato preferido usando `ffmpeg` e incrustar la carátula oficial.
3. **Escaneo de Metadatos**: El binario lee la información de la pista (artista, título) directamente de las etiquetas del archivo generado mediante la biblioteca `lofty`.
4. **Embedding de Letras**: Con los metadatos de artista y título, consulta la API pública de **LRCLIB**. Si hay letras disponibles (sincronizadas o planas), las escribe en los tags de audio nativos (`LYRICS` en FLAC y `USLT` en MP3).
