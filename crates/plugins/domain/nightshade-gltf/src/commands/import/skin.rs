use super::*;

pub(super) fn load_gltf_skin(skin: &gltf::Skin, buffers: &[buffer::Data]) -> ModelSkin {
    let joints: Vec<usize> = skin.joints().map(|j| j.index()).collect();

    let mut inverse_bind_matrices = Vec::new();

    if let Some(accessor) = skin.inverse_bind_matrices() {
        let view = accessor
            .view()
            .expect("Inverse bind matrices accessor has no view");
        let buffer_index = view.buffer().index();

        if let Some(buffer_data) = buffers.get(buffer_index) {
            let buffer_bytes = &buffer_data.0;
            let view_offset = view.offset();
            let accessor_offset = accessor.offset();
            let start = view_offset + accessor_offset;
            let stride = view.stride().unwrap_or(64);

            for joint_index in 0..accessor.count() {
                let matrix_start = start + joint_index * stride;
                if matrix_start + 64 <= buffer_bytes.len() {
                    let mut floats = [0.0f32; 16];
                    for (float_index, float_value) in floats.iter_mut().enumerate() {
                        let byte_offset = matrix_start + float_index * 4;
                        let bytes = &buffer_bytes[byte_offset..byte_offset + 4];
                        *float_value = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                    }

                    let matrix = Mat4::new(
                        floats[0], floats[4], floats[8], floats[12], floats[1], floats[5],
                        floats[9], floats[13], floats[2], floats[6], floats[10], floats[14],
                        floats[3], floats[7], floats[11], floats[15],
                    );
                    inverse_bind_matrices.push(matrix);
                }
            }
        }
    }

    while inverse_bind_matrices.len() < joints.len() {
        inverse_bind_matrices.push(Mat4::identity());
    }

    ModelSkin {
        name: skin.name().map(|s| s.to_string()),
        joints,
        inverse_bind_matrices,
    }
}
