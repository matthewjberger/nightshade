#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TexelFormat {
    #[default]
    Rgba8,
    Bc1,
    Bc3,
    Bc4,
    Bc5,
    Bc7,
    Astc4x4,
}

impl TexelFormat {
    pub fn is_compressed(self) -> bool {
        !matches!(self, TexelFormat::Rgba8)
    }

    pub fn block(self) -> (u32, usize) {
        match self {
            TexelFormat::Rgba8 => (1, 4),
            TexelFormat::Bc1 | TexelFormat::Bc4 => (4, 8),
            TexelFormat::Bc3 | TexelFormat::Bc5 | TexelFormat::Bc7 | TexelFormat::Astc4x4 => {
                (4, 16)
            }
        }
    }

    pub fn level_bytes(self, width: u32, height: u32) -> usize {
        let (side, bytes) = self.block();
        let across = width.max(1).div_ceil(side) as usize;
        let down = height.max(1).div_ceil(side) as usize;
        across * down * bytes
    }
}

#[derive(Clone)]
pub struct DecodedImage {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: TexelFormat,
    pub mips: Vec<Vec<u8>>,
}

impl DecodedImage {
    pub fn rgba8(rgba: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            rgba,
            width,
            height,
            format: TexelFormat::Rgba8,
            mips: Vec::new(),
        }
    }
}

pub fn decode_to_rgba8(encoded_bytes: &[u8]) -> Result<DecodedImage, image::ImageError> {
    let dynamic = image::load_from_memory(encoded_bytes)?;
    let width = dynamic.width();
    let height = dynamic.height();
    let rgba = dynamic.to_rgba8().into_raw();
    Ok(DecodedImage::rgba8(rgba, width, height))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BlockFamily {
    Bc,
    Astc,
    #[default]
    Texels,
}

pub fn decode_image(
    encoded_bytes: &[u8],
    family: BlockFamily,
) -> Result<DecodedImage, Box<dyn std::error::Error + Send + Sync>> {
    if encoded_bytes.starts_with(&KTX2_IDENTIFIER) {
        return decode_ktx2(encoded_bytes, family);
    }
    Ok(decode_to_rgba8(encoded_bytes)?)
}

const KTX2_IDENTIFIER: [u8; 12] = [
    0xAB, b'K', b'T', b'X', b' ', b'2', b'0', 0xBB, b'\r', b'\n', 0x1A, b'\n',
];

const KTX2_LEVEL_INDEX: usize = 80;

const KTX2_PLAIN: u32 = 0;
const KTX2_BASIS_LZ: u32 = 1;
const KTX2_ZSTD: u32 = 2;

const KTX2_MODEL_ETC1S: u8 = 163;
const KTX2_MODEL_UASTC: u8 = 166;

fn ktx2_format(vk_format: u32) -> Option<TexelFormat> {
    match vk_format {
        37 | 43 => Some(TexelFormat::Rgba8),
        131..=134 => Some(TexelFormat::Bc1),
        137 | 138 => Some(TexelFormat::Bc3),
        139 | 140 => Some(TexelFormat::Bc4),
        141 | 142 => Some(TexelFormat::Bc5),
        145 | 146 => Some(TexelFormat::Bc7),
        157 | 158 => Some(TexelFormat::Astc4x4),
        _ => None,
    }
}

type DecodeError = Box<dyn std::error::Error + Send + Sync>;

struct Ktx2Header {
    vk_format: u32,
    width: u32,
    height: u32,
    levels: u32,
    supercompression: u32,
    color_model: u8,
    global_offset: usize,
    global_length: usize,
}

struct Ktx2Level {
    offset: usize,
    length: usize,
    unpacked: usize,
    width: u32,
    height: u32,
}

fn word_at(bytes: &[u8], at: usize) -> Result<u32, DecodeError> {
    let run = bytes
        .get(at..at + 4)
        .ok_or("the KTX2 header is cut short")?;
    Ok(u32::from_le_bytes([run[0], run[1], run[2], run[3]]))
}

fn long_at(bytes: &[u8], at: usize) -> Result<u64, DecodeError> {
    let run = bytes
        .get(at..at + 8)
        .ok_or("the KTX2 level index is cut short")?;
    let mut value = [0u8; 8];
    value.copy_from_slice(run);
    Ok(u64::from_le_bytes(value))
}

fn read_ktx2_header(bytes: &[u8]) -> Result<Ktx2Header, DecodeError> {
    let depth = word_at(bytes, 28)?;
    let layers = word_at(bytes, 32)?;
    let faces = word_at(bytes, 36)?;
    if depth > 1 || layers > 1 || faces != 1 {
        return Err(
            "a KTX2 file with layers, faces, or depth is not a texture this engine reads".into(),
        );
    }
    let descriptor_offset = word_at(bytes, 48)? as usize;
    let color_model = *bytes
        .get(descriptor_offset + 12)
        .ok_or("the KTX2 data format descriptor is cut short")?;
    Ok(Ktx2Header {
        vk_format: word_at(bytes, 12)?,
        width: word_at(bytes, 20)?,
        height: word_at(bytes, 24)?,
        levels: word_at(bytes, 40)?.max(1),
        supercompression: word_at(bytes, 44)?,
        color_model,
        global_offset: long_at(bytes, 64)? as usize,
        global_length: long_at(bytes, 72)? as usize,
    })
}

fn read_ktx2_levels(bytes: &[u8], header: &Ktx2Header) -> Result<Vec<Ktx2Level>, DecodeError> {
    (0..header.levels)
        .map(|level| {
            let entry = KTX2_LEVEL_INDEX + level as usize * 24;
            Ok(Ktx2Level {
                offset: long_at(bytes, entry)? as usize,
                length: long_at(bytes, entry + 8)? as usize,
                unpacked: long_at(bytes, entry + 16)? as usize,
                width: (header.width >> level).max(1),
                height: (header.height >> level).max(1),
            })
        })
        .collect()
}

fn ktx2_level_bytes(
    bytes: &[u8],
    header: &Ktx2Header,
    level: &Ktx2Level,
) -> Result<Vec<u8>, DecodeError> {
    let run = bytes
        .get(level.offset..level.offset + level.length)
        .ok_or("a KTX2 level is cut short")?;
    match header.supercompression {
        KTX2_PLAIN | KTX2_BASIS_LZ => Ok(run.to_vec()),
        KTX2_ZSTD => {
            let mut unpacked = Vec::with_capacity(level.unpacked);
            ruzstd::decoding::FrameDecoder::new()
                .decode_all_to_vec(run, &mut unpacked)
                .map_err(|error| format!("a KTX2 level did not unpack: {error}"))?;
            Ok(unpacked)
        }
        other => Err(format!(
            "a KTX2 file packed with scheme {other} is not one this engine reads"
        )
        .into()),
    }
}

fn decode_ktx2(bytes: &[u8], family: BlockFamily) -> Result<DecodedImage, DecodeError> {
    let header = read_ktx2_header(bytes)?;
    let levels = read_ktx2_levels(bytes, &header)?;
    let (format, carried) = if header.vk_format == 0 {
        transcode_ktx2(bytes, &header, &levels, family)?
    } else {
        if family == BlockFamily::Texels {
            return Err("a KTX2 file that names a block format cannot be read as texels".into());
        }
        let format = ktx2_format(header.vk_format).ok_or_else(|| {
            format!(
                "the KTX2 format {} is not one this engine reads",
                header.vk_format
            )
        })?;
        let mut carried = Vec::with_capacity(levels.len());
        for level in &levels {
            let wanted = format.level_bytes(level.width, level.height);
            let run = ktx2_level_bytes(bytes, &header, level)?;
            if run.len() < wanted {
                return Err("a KTX2 level is cut short".into());
            }
            carried.push(run[..wanted].to_vec());
        }
        (format, carried)
    };
    let mut carried = carried.into_iter();
    let rgba = carried.next().ok_or("the KTX2 file carries no level")?;
    Ok(DecodedImage {
        rgba,
        width: header.width,
        height: header.height,
        format,
        mips: match family {
            BlockFamily::Bc | BlockFamily::Astc => carried.collect(),
            BlockFamily::Texels => Vec::new(),
        },
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn transcode_ktx2(
    bytes: &[u8],
    header: &Ktx2Header,
    levels: &[Ktx2Level],
    family: BlockFamily,
) -> Result<(TexelFormat, Vec<Vec<u8>>), DecodeError> {
    basis_universal::transcoder_init();
    let (container, with_alpha) = basis_container_of(bytes, header, levels)?;
    let (format, texture_format) = match (family, header.color_model, with_alpha) {
        (BlockFamily::Bc, KTX2_MODEL_ETC1S, false) => (
            TexelFormat::Bc1,
            basis_universal::TranscoderTextureFormat::BC1_RGB,
        ),
        (BlockFamily::Bc, KTX2_MODEL_ETC1S, true) => (
            TexelFormat::Bc3,
            basis_universal::TranscoderTextureFormat::BC3_RGBA,
        ),
        (BlockFamily::Bc, _, _) => (
            TexelFormat::Bc7,
            basis_universal::TranscoderTextureFormat::BC7_RGBA,
        ),
        (BlockFamily::Astc, _, _) => (
            TexelFormat::Astc4x4,
            basis_universal::TranscoderTextureFormat::ASTC_4x4_RGBA,
        ),
        (BlockFamily::Texels, _, _) => (
            TexelFormat::Rgba8,
            basis_universal::TranscoderTextureFormat::RGBA32,
        ),
    };
    let mut transcoder = basis_universal::Transcoder::new();
    transcoder
        .prepare_transcoding(&container)
        .map_err(|()| "the universal texture did not prepare to transcode")?;
    let carried = (0..levels.len() as u32)
        .map(|level| {
            transcoder
                .transcode_image_level(
                    &container,
                    texture_format,
                    basis_universal::TranscodeParameters {
                        image_index: 0,
                        level_index: level,
                        decode_flags: None,
                        output_row_pitch_in_blocks_or_pixels: None,
                        output_rows_in_pixels: None,
                    },
                )
                .map_err(|error| {
                    format!("a universal texture level did not transcode: {error:?}").into()
                })
        })
        .collect::<Result<Vec<Vec<u8>>, DecodeError>>();
    transcoder.end_transcoding();
    Ok((format, carried?))
}

#[cfg(target_arch = "wasm32")]
fn transcode_ktx2(
    bytes: &[u8],
    header: &Ktx2Header,
    levels: &[Ktx2Level],
    family: BlockFamily,
) -> Result<(TexelFormat, Vec<Vec<u8>>), DecodeError> {
    let (container, _) = basis_container_of(bytes, header, levels)?;
    Err(format!(
        "a universal texture of {} bytes needs the native transcoder to come out as {:?}, which a web build does not carry",
        container.len(),
        family
    )
    .into())
}

const BASIS_HEADER_SIZE: usize = 77;
const BASIS_SLICE_SIZE: usize = 23;
const BASIS_SIGNATURE: u16 = (b'B' as u16) << 8 | b's' as u16;
const BASIS_VERSION: u16 = 0x13;
const BASIS_FORMAT_ETC1S: u8 = 0;
const BASIS_FORMAT_UASTC: u8 = 1;
const BASIS_FLAG_ETC1S: u16 = 1;
const BASIS_FLAG_HAS_ALPHA_SLICES: u16 = 4;
const BASIS_SLICE_HAS_ALPHA: u8 = 1;
const KTX2_GLOBAL_HEADER: usize = 20;
const KTX2_IMAGE_DESCRIPTION: usize = 20;

fn basis_crc16(bytes: &[u8]) -> u16 {
    let mut crc: u16 = !0;
    for byte in bytes {
        let mixed = u16::from(*byte) ^ (crc >> 8);
        let folded = (mixed >> 4) ^ mixed;
        crc = ((crc << 8) ^ folded) ^ (folded << 5) ^ (folded << 12);
    }
    !crc
}

fn put_packed(out: &mut Vec<u8>, value: u64, bytes: usize) {
    out.extend_from_slice(&value.to_le_bytes()[..bytes]);
}

struct BasisSlice {
    level: u32,
    has_alpha: bool,
    data: Vec<u8>,
}

#[derive(Default)]
struct BasisShared {
    endpoint_count: u16,
    selector_count: u16,
    endpoints: usize,
    selectors: usize,
    tables: usize,
    extended: usize,
    bytes: Vec<u8>,
}

fn etc1s_slices_of(
    bytes: &[u8],
    header: &Ktx2Header,
    levels: &[Ktx2Level],
) -> Result<(BasisShared, Vec<BasisSlice>), DecodeError> {
    if header.supercompression != KTX2_BASIS_LZ {
        return Err("an ETC1S KTX2 file must carry its codebooks".into());
    }
    let global = bytes
        .get(header.global_offset..header.global_offset + header.global_length)
        .ok_or("the KTX2 global data is cut short")?;
    let endpoints = word_at(global, 4)? as usize;
    let selectors = word_at(global, 8)? as usize;
    let tables = word_at(global, 12)? as usize;
    let extended = word_at(global, 16)? as usize;
    let codebooks = KTX2_GLOBAL_HEADER + KTX2_IMAGE_DESCRIPTION * levels.len();
    let shared = BasisShared {
        endpoint_count: u16::from_le_bytes([global[0], global[1]]),
        selector_count: u16::from_le_bytes([global[2], global[3]]),
        endpoints,
        selectors,
        tables,
        extended,
        bytes: global
            .get(codebooks..codebooks + endpoints + selectors + tables + extended)
            .ok_or("the KTX2 global data is cut short")?
            .to_vec(),
    };

    let mut slices = Vec::with_capacity(levels.len() * 2);
    let mut with_alpha = 0;
    for (index, level) in levels.iter().enumerate() {
        let description = KTX2_GLOBAL_HEADER + KTX2_IMAGE_DESCRIPTION * index;
        let color_offset = word_at(global, description + 4)? as usize;
        let color_length = word_at(global, description + 8)? as usize;
        let alpha_offset = word_at(global, description + 12)? as usize;
        let alpha_length = word_at(global, description + 16)? as usize;
        let run = bytes
            .get(level.offset..level.offset + level.length)
            .ok_or("a KTX2 level is cut short")?;
        let color = run
            .get(color_offset..color_offset + color_length)
            .ok_or("a KTX2 color slice is cut short")?;
        slices.push(BasisSlice {
            level: index as u32,
            has_alpha: false,
            data: color.to_vec(),
        });
        if alpha_length > 0 {
            let alpha = run
                .get(alpha_offset..alpha_offset + alpha_length)
                .ok_or("a KTX2 alpha slice is cut short")?;
            slices.push(BasisSlice {
                level: index as u32,
                has_alpha: true,
                data: alpha.to_vec(),
            });
            with_alpha += 1;
        }
    }
    if with_alpha != 0 && with_alpha != levels.len() {
        return Err("a KTX2 file holds an alpha slice for some levels and not others".into());
    }
    Ok((shared, slices))
}

fn uastc_slices_of(
    bytes: &[u8],
    header: &Ktx2Header,
    levels: &[Ktx2Level],
) -> Result<Vec<BasisSlice>, DecodeError> {
    levels
        .iter()
        .enumerate()
        .map(|(index, level)| {
            Ok(BasisSlice {
                level: index as u32,
                has_alpha: true,
                data: ktx2_level_bytes(bytes, header, level)?,
            })
        })
        .collect()
}

fn basis_container_of(
    bytes: &[u8],
    header: &Ktx2Header,
    levels: &[Ktx2Level],
) -> Result<(Vec<u8>, bool), DecodeError> {
    let (texture_format, shared, slices) = match header.color_model {
        KTX2_MODEL_ETC1S => {
            let (shared, slices) = etc1s_slices_of(bytes, header, levels)?;
            (BASIS_FORMAT_ETC1S, shared, slices)
        }
        KTX2_MODEL_UASTC => (
            BASIS_FORMAT_UASTC,
            BasisShared::default(),
            uastc_slices_of(bytes, header, levels)?,
        ),
        other => {
            return Err(
                format!("the KTX2 color model {other} is not one this engine reads").into(),
            );
        }
    };
    let any_alpha = slices.iter().any(|slice| slice.has_alpha);

    let descriptions_at = BASIS_HEADER_SIZE + shared.bytes.len();
    let slice_data_at = descriptions_at + BASIS_SLICE_SIZE * slices.len();
    let data_size = shared.bytes.len()
        + BASIS_SLICE_SIZE * slices.len()
        + slices.iter().map(|slice| slice.data.len()).sum::<usize>();
    let mut container = Vec::with_capacity(BASIS_HEADER_SIZE + data_size);
    container.resize(BASIS_HEADER_SIZE, 0);
    container.extend_from_slice(&shared.bytes);
    let mut slice_at = slice_data_at;
    for slice in &slices {
        let level = &levels[slice.level as usize];
        put_packed(&mut container, 0, 3);
        put_packed(&mut container, u64::from(slice.level), 1);
        put_packed(
            &mut container,
            u64::from(if slice.has_alpha {
                BASIS_SLICE_HAS_ALPHA
            } else {
                0
            }),
            1,
        );
        put_packed(&mut container, u64::from(level.width), 2);
        put_packed(&mut container, u64::from(level.height), 2);
        put_packed(&mut container, u64::from(level.width.div_ceil(4)), 2);
        put_packed(&mut container, u64::from(level.height.div_ceil(4)), 2);
        put_packed(&mut container, slice_at as u64, 4);
        put_packed(&mut container, slice.data.len() as u64, 4);
        put_packed(&mut container, u64::from(basis_crc16(&slice.data)), 2);
        slice_at += slice.data.len();
    }
    for slice in &slices {
        container.extend_from_slice(&slice.data);
    }

    let endpoints_at = BASIS_HEADER_SIZE;
    let selectors_at = endpoints_at + shared.endpoints;
    let tables_at = selectors_at + shared.selectors;
    let extended_at = tables_at + shared.tables;
    let flags = if texture_format == BASIS_FORMAT_ETC1S {
        BASIS_FLAG_ETC1S
    } else {
        0
    } | if any_alpha {
        BASIS_FLAG_HAS_ALPHA_SLICES
    } else {
        0
    };
    let mut head = Vec::with_capacity(BASIS_HEADER_SIZE);
    put_packed(&mut head, u64::from(BASIS_SIGNATURE), 2);
    put_packed(&mut head, u64::from(BASIS_VERSION), 2);
    put_packed(&mut head, BASIS_HEADER_SIZE as u64, 2);
    put_packed(&mut head, 0, 2);
    put_packed(&mut head, data_size as u64, 4);
    put_packed(
        &mut head,
        u64::from(basis_crc16(&container[BASIS_HEADER_SIZE..])),
        2,
    );
    put_packed(&mut head, slices.len() as u64, 3);
    put_packed(&mut head, 1, 3);
    put_packed(&mut head, u64::from(texture_format), 1);
    put_packed(&mut head, u64::from(flags), 2);
    put_packed(&mut head, 0, 1);
    put_packed(&mut head, 0, 3);
    put_packed(&mut head, 0, 4);
    put_packed(&mut head, 0, 4);
    put_packed(&mut head, 0, 4);
    put_packed(&mut head, u64::from(shared.endpoint_count), 2);
    put_packed(&mut head, endpoints_at as u64, 4);
    put_packed(&mut head, shared.endpoints as u64, 3);
    put_packed(&mut head, u64::from(shared.selector_count), 2);
    put_packed(&mut head, selectors_at as u64, 4);
    put_packed(&mut head, shared.selectors as u64, 3);
    put_packed(&mut head, tables_at as u64, 4);
    put_packed(&mut head, shared.tables as u64, 4);
    put_packed(&mut head, descriptions_at as u64, 4);
    put_packed(&mut head, extended_at as u64, 4);
    put_packed(&mut head, shared.extended as u64, 4);
    let checked = basis_crc16(&head[8..]);
    head[6..8].copy_from_slice(&checked.to_le_bytes());
    container[..BASIS_HEADER_SIZE].copy_from_slice(&head);
    Ok((container, any_alpha))
}
