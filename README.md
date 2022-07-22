# hdrcopier

## Dependencies

- mkvtoolnix CLI
- mediainfo CLI
- ffprobe

## Usage

### Copy metadata

`hdrcopier copy [input] [target]`

- input = file to copy metadata from
- target = file to copy metadata into

The tool will parse the metadata from the input file,
then update the target file with that metadata.

### Display metadata

`hdrcopier show [input]`

Will display metadata on the screen, by default in a human-readable format.

Optionally, a `--format` flag can be passed to format the metadata to be passed
directly to an encoder.

## Bugs

If you have a video that you know is HDR, but this tool fails to parse the metadata,
please create an issue. If you can include a link to the video, that's extremely
helpful. If not, please post the _full CLI output_ from both mkvinfo and mediainfo
for the file.
