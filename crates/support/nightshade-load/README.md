# nightshade-load

This crate reads bytes off the frame. A caller declares what a kind of asset is by implementing `Reader`, inserts a `Loader<R>` as a resource, and asks for a source. The read runs on a `nightshade-worker` crew, so a large file never stalls the frame that asked for it, and behind the `fetch` feature a URL is read by the browser or by ehttp instead. The crate knows no file format of its own.

## Composing it

The crate carries no plugin. The owner of an asset kind inserts its own `Loader<R>` as a resource, which is what `nightshade-gltf` does with `pub type Loading = Loader<Gltf>`.

## Using it

```rust
use nightshade_load::{Reader, Source, bytes_of, load, load_noted};

impl Reader for Picture {
    type Item = DecodedImage;
    type Note = ();

    fn read(source: Source) -> Result<DecodedImage, String> {
        let bytes = bytes_of(source)?;
        decode_image(&bytes, BlockFamily::Texels)
            .map_err(|error| format!("a picture did not decode: {error}"))
    }
}

let handle = load(pictures, Source::Bytes(theme::SHEET.to_vec()));
```

`Reader::read` runs on a worker thread and turns a `Source` into the reader's `Item`, and `bytes_of` is the helper that flattens a path or an inline buffer into bytes. `Source` is a path, a buffer, a head with its side files, or a URL under `fetch`. `load` hands back a `LoadHandle` at once, `load_noted` attaches a `Note` the caller gets back when the read lands, and `Reader::hands` says how many workers the crew hires. Each frame the owner calls `gather` with a limit to move what finished into the ready table, `ready` asks whether one handle has landed, `in_flight` counts what is still being read, and `take` removes the result.

## Taking it from another crate

The crate carries no `contract` feature, because the whole crate is the contract. It depends on `nightshade-worker` alone, names no ECS and no engine, so a crate that reads assets takes it whole.
