# hdrcopier

## Dependencies

- mkvtoolnix CLI
- mediainfo CLI

## Usage

`hdrcopier [input] [target] [output]`

- input = file to copy metadata from
- target = file to copy metadata to
- output = location to place the resulting file

To clarify:
The tool does not overwrite the `input` or `target` files,
it takes the metadata from `input`,
and the media streams from `target`,
and muxes them together into `output`.
