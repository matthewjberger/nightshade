use super::*;

pub(super) fn load_gltf_animation(
    animation: &gltf::Animation,
    buffers: &[buffer::Data],
    node_to_morph_target_count: &HashMap<usize, usize>,
) -> AnimationClip {
    let mut channels = Vec::new();

    for (channel_index, channel) in animation.channels().enumerate() {
        let reader = channel.reader(|buffer| buffers.get(buffer.index()).map(|b| &b.0[..]));

        let target_gltf_node = channel.target().node();
        let target_node = target_gltf_node.index();
        let target_bone_name = target_gltf_node.name().map(|s| s.to_string());
        let target_property = match channel.target().property() {
            gltf::animation::Property::Translation => AnimationProperty::Translation,
            gltf::animation::Property::Rotation => AnimationProperty::Rotation,
            gltf::animation::Property::Scale => AnimationProperty::Scale,
            gltf::animation::Property::MorphTargetWeights => AnimationProperty::MorphWeights,
        };

        let interpolation = match channel.sampler().interpolation() {
            gltf::animation::Interpolation::Linear => AnimationInterpolation::Linear,
            gltf::animation::Interpolation::Step => AnimationInterpolation::Step,
            gltf::animation::Interpolation::CubicSpline => AnimationInterpolation::CubicSpline,
        };

        let Some(inputs) = reader.read_inputs() else {
            tracing::warn!("  Channel {}: skipping - no input data", channel_index);
            continue;
        };
        let input: Vec<f32> = inputs.collect();

        let Some(outputs) = reader.read_outputs() else {
            tracing::warn!("  Channel {}: skipping - no output data", channel_index);
            continue;
        };

        let output = match outputs {
            gltf::animation::util::ReadOutputs::Translations(translations) => {
                if interpolation == AnimationInterpolation::CubicSpline {
                    let all_data: Vec<[f32; 3]> = translations.collect();
                    let num_keyframes = all_data.len() / 3;

                    let mut in_tangents = Vec::with_capacity(num_keyframes);
                    let mut values = Vec::with_capacity(num_keyframes);
                    let mut out_tangents = Vec::with_capacity(num_keyframes);

                    for keyframe_index in 0..num_keyframes {
                        let base_idx = keyframe_index * 3;
                        in_tangents.push(vec3(
                            all_data[base_idx][0],
                            all_data[base_idx][1],
                            all_data[base_idx][2],
                        ));
                        values.push(vec3(
                            all_data[base_idx + 1][0],
                            all_data[base_idx + 1][1],
                            all_data[base_idx + 1][2],
                        ));
                        out_tangents.push(vec3(
                            all_data[base_idx + 2][0],
                            all_data[base_idx + 2][1],
                            all_data[base_idx + 2][2],
                        ));
                    }

                    AnimationSamplerOutput::CubicSplineVec3 {
                        values,
                        in_tangents,
                        out_tangents,
                    }
                } else {
                    let values = translations.map(|t| vec3(t[0], t[1], t[2])).collect();
                    AnimationSamplerOutput::Vec3(values)
                }
            }
            gltf::animation::util::ReadOutputs::Rotations(rotations) => {
                if interpolation == AnimationInterpolation::CubicSpline {
                    let all_data: Vec<[f32; 4]> = rotations.into_f32().collect();
                    let num_keyframes = all_data.len() / 3;

                    let mut in_tangents = Vec::with_capacity(num_keyframes);
                    let mut values = Vec::with_capacity(num_keyframes);
                    let mut out_tangents = Vec::with_capacity(num_keyframes);

                    for keyframe_index in 0..num_keyframes {
                        let base_idx = keyframe_index * 3;
                        in_tangents.push(Quat::new(
                            all_data[base_idx][3],
                            all_data[base_idx][0],
                            all_data[base_idx][1],
                            all_data[base_idx][2],
                        ));
                        values.push(
                            Quat::new(
                                all_data[base_idx + 1][3],
                                all_data[base_idx + 1][0],
                                all_data[base_idx + 1][1],
                                all_data[base_idx + 1][2],
                            )
                            .normalize(),
                        );
                        out_tangents.push(Quat::new(
                            all_data[base_idx + 2][3],
                            all_data[base_idx + 2][0],
                            all_data[base_idx + 2][1],
                            all_data[base_idx + 2][2],
                        ));
                    }

                    AnimationSamplerOutput::CubicSplineQuat {
                        values,
                        in_tangents,
                        out_tangents,
                    }
                } else {
                    let values = rotations
                        .into_f32()
                        .map(|r| Quat::new(r[3], r[0], r[1], r[2]).normalize())
                        .collect();
                    AnimationSamplerOutput::Quat(values)
                }
            }
            gltf::animation::util::ReadOutputs::Scales(scales) => {
                if interpolation == AnimationInterpolation::CubicSpline {
                    let all_data: Vec<[f32; 3]> = scales.collect();
                    let num_keyframes = all_data.len() / 3;

                    let mut in_tangents = Vec::with_capacity(num_keyframes);
                    let mut values = Vec::with_capacity(num_keyframes);
                    let mut out_tangents = Vec::with_capacity(num_keyframes);

                    for keyframe_index in 0..num_keyframes {
                        let base_idx = keyframe_index * 3;
                        in_tangents.push(vec3(
                            all_data[base_idx][0],
                            all_data[base_idx][1],
                            all_data[base_idx][2],
                        ));
                        values.push(vec3(
                            all_data[base_idx + 1][0],
                            all_data[base_idx + 1][1],
                            all_data[base_idx + 1][2],
                        ));
                        out_tangents.push(vec3(
                            all_data[base_idx + 2][0],
                            all_data[base_idx + 2][1],
                            all_data[base_idx + 2][2],
                        ));
                    }

                    AnimationSamplerOutput::CubicSplineVec3 {
                        values,
                        in_tangents,
                        out_tangents,
                    }
                } else {
                    let values = scales.map(|s| vec3(s[0], s[1], s[2])).collect();
                    AnimationSamplerOutput::Vec3(values)
                }
            }
            gltf::animation::util::ReadOutputs::MorphTargetWeights(weights) => {
                let weight_count = node_to_morph_target_count
                    .get(&target_node)
                    .copied()
                    .unwrap_or(1);
                if weight_count == 0 {
                    continue;
                }

                if interpolation == AnimationInterpolation::CubicSpline {
                    let all_data: Vec<f32> = weights.into_f32().collect();
                    let elements_per_keyframe = weight_count * 3;
                    if !all_data.len().is_multiple_of(elements_per_keyframe) {
                        continue;
                    }
                    let num_keyframes = all_data.len() / elements_per_keyframe;

                    let mut in_tangents = Vec::with_capacity(num_keyframes);
                    let mut values = Vec::with_capacity(num_keyframes);
                    let mut out_tangents = Vec::with_capacity(num_keyframes);

                    for keyframe_index in 0..num_keyframes {
                        let base_idx = keyframe_index * elements_per_keyframe;
                        let in_tangent: Vec<f32> =
                            all_data[base_idx..base_idx + weight_count].to_vec();
                        let value: Vec<f32> =
                            all_data[base_idx + weight_count..base_idx + weight_count * 2].to_vec();
                        let out_tangent: Vec<f32> = all_data
                            [base_idx + weight_count * 2..base_idx + weight_count * 3]
                            .to_vec();
                        in_tangents.push(in_tangent);
                        values.push(value);
                        out_tangents.push(out_tangent);
                    }

                    AnimationSamplerOutput::CubicSplineWeights {
                        values,
                        in_tangents,
                        out_tangents,
                    }
                } else {
                    let all_data: Vec<f32> = weights.into_f32().collect();
                    if !all_data.len().is_multiple_of(weight_count) {
                        continue;
                    }
                    let values: Vec<Vec<f32>> = all_data
                        .chunks(weight_count)
                        .map(|chunk| chunk.to_vec())
                        .collect();
                    AnimationSamplerOutput::Weights(values)
                }
            }
        };

        let sampler = AnimationSampler {
            input,
            output,
            interpolation,
        };

        channels.push(AnimationChannel {
            target_node,
            target_bone_name,
            target_property,
            sampler,
        });
    }

    let duration = channels
        .iter()
        .flat_map(|channel| channel.sampler.input.last())
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .copied()
        .unwrap_or(0.0);

    AnimationClip {
        name: animation.name().unwrap_or("Animation").to_string(),
        duration,
        channels,
        events: Vec::new(),
        source: 0,
    }
}
