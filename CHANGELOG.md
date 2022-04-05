## Version 0.3.1

- Fix more bugs with color matrix setting

## Version 0.3.0

- [Breaking] Rework how the remuxing works to use mkvtoolnix's header editor
  - This avoids the need for remuxing and also fixes apparently a massive bug with muxing color primaries, etc.
  - This means that the need for the third `output` parameter is removed.

## Version 0.2.2

- Try ffprobe before mediainfo because it's faster and more likely to have the HDR info

## Version 0.2.1

- Fix for certain streams where mediainfo parsed lighting data but not mastering display data

## Version 0.2.0

- [Breaking] Move `hdrcopier` copy behavior to `hdrcopier copy`
- Add `hdrcopier show` command

## Version 0.1.1

- Add ffprobe as a tertiary fallback for probing HDR metadata

## Version 0.1.0

- Initial release
