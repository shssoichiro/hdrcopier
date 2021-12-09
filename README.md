# hdrcopier

## Dependencies

- mkvtoolnix CLI
- mediainfo CLI
- ffprobe

## Usage

### Copy metadata

`hdrcopier copy [input] [target] [output]`

- input = file to copy metadata from
- target = file to copy metadata to
- output = location to place the resulting file

To clarify:
The tool does not overwrite the `input` or `target` files,
it takes the metadata from `input`,
and the media streams from `target`,
and muxes them together into `output`.

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
