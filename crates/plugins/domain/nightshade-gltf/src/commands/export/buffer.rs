use serde_json::{Value, json};

pub const ARRAY_BUFFER: u32 = 34962;
pub const ELEMENT_ARRAY_BUFFER: u32 = 34963;

pub const FLOAT: u32 = 5126;
pub const UNSIGNED_INT: u32 = 5125;
pub const UNSIGNED_SHORT: u32 = 5123;

#[derive(Default)]
pub struct BufferBuilder {
    pub bytes: Vec<u8>,
    pub views: Vec<Value>,
    pub accessors: Vec<Value>,
}

impl BufferBuilder {
    fn align(&mut self) {
        while !self.bytes.len().is_multiple_of(4) {
            self.bytes.push(0);
        }
    }

    fn view(&mut self, data: &[u8], stride: Option<u32>, target: Option<u32>) -> usize {
        self.align();
        let offset = self.bytes.len();
        self.bytes.extend_from_slice(data);
        let mut view = json!({
            "buffer": 0,
            "byteOffset": offset,
            "byteLength": data.len(),
        });
        if let Some(stride) = stride {
            view["byteStride"] = json!(stride);
        }
        if let Some(target) = target {
            view["target"] = json!(target);
        }
        self.views.push(view);
        self.views.len() - 1
    }

    fn accessor(&mut self, accessor: Value) -> usize {
        self.accessors.push(accessor);
        self.accessors.len() - 1
    }

    pub fn floats(&mut self, values: &[f32], kind: &str, target: Option<u32>) -> usize {
        let count = values.len() / components(kind);
        let mut data = Vec::with_capacity(values.len() * 4);
        for value in values {
            data.extend_from_slice(&value.to_le_bytes());
        }
        let view = self.view(&data, None, target);
        let mut accessor = json!({
            "bufferView": view,
            "componentType": FLOAT,
            "count": count,
            "type": kind,
        });
        if kind == "VEC3" {
            let (minimum, maximum) = bounds(values, 3);
            accessor["min"] = json!(minimum);
            accessor["max"] = json!(maximum);
        } else if kind == "SCALAR" {
            let (minimum, maximum) = bounds(values, 1);
            accessor["min"] = json!(minimum);
            accessor["max"] = json!(maximum);
        }
        self.accessor(accessor)
    }

    pub fn indices(&mut self, values: &[u32]) -> usize {
        let mut data = Vec::with_capacity(values.len() * 4);
        for value in values {
            data.extend_from_slice(&value.to_le_bytes());
        }
        let view = self.view(&data, None, Some(ELEMENT_ARRAY_BUFFER));
        self.accessor(json!({
            "bufferView": view,
            "componentType": UNSIGNED_INT,
            "count": values.len(),
            "type": "SCALAR",
        }))
    }

    pub fn joints(&mut self, values: &[[u32; 4]]) -> usize {
        let mut data = Vec::with_capacity(values.len() * 8);
        for joint in values {
            for index in joint {
                let narrowed = (*index).min(u16::MAX as u32) as u16;
                data.extend_from_slice(&narrowed.to_le_bytes());
            }
        }
        let view = self.view(&data, Some(8), Some(ARRAY_BUFFER));
        self.accessor(json!({
            "bufferView": view,
            "componentType": UNSIGNED_SHORT,
            "count": values.len(),
            "type": "VEC4",
        }))
    }

    pub fn matrices(&mut self, values: &[[f32; 16]]) -> usize {
        let mut data = Vec::with_capacity(values.len() * 64);
        for matrix in values {
            for value in matrix {
                data.extend_from_slice(&value.to_le_bytes());
            }
        }
        let view = self.view(&data, None, None);
        self.accessor(json!({
            "bufferView": view,
            "componentType": FLOAT,
            "count": values.len(),
            "type": "MAT4",
        }))
    }

    pub fn image(&mut self, png: &[u8]) -> usize {
        self.view(png, None, None)
    }
}

fn components(kind: &str) -> usize {
    match kind {
        "SCALAR" => 1,
        "VEC2" => 2,
        "VEC3" => 3,
        "VEC4" => 4,
        _ => 1,
    }
}

fn bounds(values: &[f32], stride: usize) -> (Vec<f32>, Vec<f32>) {
    let mut minimum = vec![f32::INFINITY; stride];
    let mut maximum = vec![f32::NEG_INFINITY; stride];
    for chunk in values.chunks_exact(stride) {
        for (index, value) in chunk.iter().enumerate() {
            minimum[index] = minimum[index].min(*value);
            maximum[index] = maximum[index].max(*value);
        }
    }
    if values.is_empty() {
        minimum = vec![0.0; stride];
        maximum = vec![0.0; stride];
    }
    (minimum, maximum)
}
