
//! Raw compiler bindings for SPIRV-Cross.

use bindings::root::*;
use ErrorCode;
use spirv;
use spirv::Decoration;
use std::{mem, ptr, slice};
use std::ffi::CStr;

impl spirv::ExecutionModel {
    fn from_raw(raw: spv::ExecutionModel) -> Result<Self, ErrorCode> {
        use spirv::ExecutionModel::*;
        use self::spv::ExecutionModel as Em;
        match raw {
            Em::ExecutionModelVertex => Ok(Vertex),
            Em::ExecutionModelTessellationControl => Ok(TessellationControl),
            Em::ExecutionModelTessellationEvaluation => Ok(TessellationEvaluation),
            Em::ExecutionModelGeometry => Ok(Geometry),
            Em::ExecutionModelFragment => Ok(Fragment),
            Em::ExecutionModelGLCompute => Ok(GlCompute),
            Em::ExecutionModelKernel => Ok(Kernel),
            _ => Err(ErrorCode::Unhandled),
        }
    }

    pub(crate) fn as_raw(&self) -> spv::ExecutionModel {
        use spirv::ExecutionModel::*;
        use self::spv::ExecutionModel as Em;
        match *self {
            Vertex => Em::ExecutionModelVertex,
            TessellationControl => Em::ExecutionModelTessellationControl,
            TessellationEvaluation => Em::ExecutionModelTessellationEvaluation,
            Geometry => Em::ExecutionModelGeometry,
            Fragment => Em::ExecutionModelFragment,
            GlCompute => Em::ExecutionModelGLCompute,
            Kernel => Em::ExecutionModelKernel,
        }
    }
}

impl spirv::Decoration {
    fn as_raw(&self) -> spv::Decoration {
        match *self {
            Decoration::RelaxedPrecision => spv::Decoration::DecorationRelaxedPrecision,
            Decoration::SpecId => spv::Decoration::DecorationSpecId,
            Decoration::Block => spv::Decoration::DecorationBlock,
            Decoration::BufferBlock => spv::Decoration::DecorationBufferBlock,
            Decoration::RowMajor => spv::Decoration::DecorationRowMajor,
            Decoration::ColMajor => spv::Decoration::DecorationColMajor,
            Decoration::ArrayStride => spv::Decoration::DecorationArrayStride,
            Decoration::MatrixStride => spv::Decoration::DecorationMatrixStride,
            Decoration::GlslShared => spv::Decoration::DecorationGLSLShared,
            Decoration::GlslPacked => spv::Decoration::DecorationGLSLPacked,
            Decoration::CPacked => spv::Decoration::DecorationCPacked,
            Decoration::BuiltIn => spv::Decoration::DecorationBuiltIn,
            Decoration::NoPerspective => spv::Decoration::DecorationNoPerspective,
            Decoration::Flat => spv::Decoration::DecorationFlat,
            Decoration::Patch => spv::Decoration::DecorationPatch,
            Decoration::Centroid => spv::Decoration::DecorationCentroid,
            Decoration::Sample => spv::Decoration::DecorationSample,
            Decoration::Invariant => spv::Decoration::DecorationInvariant,
            Decoration::Restrict => spv::Decoration::DecorationRestrict,
            Decoration::Aliased => spv::Decoration::DecorationAliased,
            Decoration::Volatile => spv::Decoration::DecorationVolatile,
            Decoration::Constant => spv::Decoration::DecorationConstant,
            Decoration::Coherent => spv::Decoration::DecorationCoherent,
            Decoration::NonWritable => spv::Decoration::DecorationNonWritable,
            Decoration::NonReadable => spv::Decoration::DecorationNonReadable,
            Decoration::Uniform => spv::Decoration::DecorationUniform,
            Decoration::SaturatedConversion => spv::Decoration::DecorationSaturatedConversion,
            Decoration::Stream => spv::Decoration::DecorationStream,
            Decoration::Location => spv::Decoration::DecorationLocation,
            Decoration::Component => spv::Decoration::DecorationComponent,
            Decoration::Index => spv::Decoration::DecorationIndex,
            Decoration::Binding => spv::Decoration::DecorationBinding,
            Decoration::DescriptorSet => spv::Decoration::DecorationDescriptorSet,
            Decoration::Offset => spv::Decoration::DecorationOffset,
            Decoration::XfbBuffer => spv::Decoration::DecorationXfbBuffer,
            Decoration::XfbStride => spv::Decoration::DecorationXfbStride,
            Decoration::FuncParamAttr => spv::Decoration::DecorationFuncParamAttr,
            Decoration::FpRoundingMode => spv::Decoration::DecorationFPRoundingMode,
            Decoration::FpFastMathMode => spv::Decoration::DecorationFPFastMathMode,
            Decoration::LinkageAttributes => spv::Decoration::DecorationLinkageAttributes,
            Decoration::NoContraction => spv::Decoration::DecorationNoContraction,
            Decoration::InputAttachmentIndex => spv::Decoration::DecorationInputAttachmentIndex,
            Decoration::Alignment => spv::Decoration::DecorationAlignment,
            Decoration::OverrideCoverageNv => spv::Decoration::DecorationOverrideCoverageNV,
            Decoration::PassthroughNv => spv::Decoration::DecorationPassthroughNV,
            Decoration::ViewportRelativeNv => spv::Decoration::DecorationViewportRelativeNV,
            Decoration::SecondaryViewportRelativeNv => {
                spv::Decoration::DecorationSecondaryViewportRelativeNV
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Compiler {
    pub sc_compiler: *mut ScInternalCompilerBase,
}

impl Compiler {
    pub fn compile(&self) -> Result<String, ErrorCode> {
        unsafe {
            let mut shader_ptr = ptr::null();
            check!(sc_internal_compiler_compile(
                self.sc_compiler,
                &mut shader_ptr,
            ));
            let shader = match CStr::from_ptr(shader_ptr).to_owned().into_string() {
                Err(_) => return Err(ErrorCode::Unhandled),
                Ok(v) => v,
            };
            check!(sc_internal_free_pointer(shader_ptr as *mut c_void));
            Ok(shader)
        }
    }

    pub fn get_decoration(&self, id: u32, decoration: spirv::Decoration) -> Result<u32, ErrorCode> {
        let mut result = 0;
        unsafe {
            check!(sc_internal_compiler_get_decoration(
                self.sc_compiler,
                &mut result,
                id,
                decoration.as_raw(),
            ));
        }
        Ok(result)
    }

    pub fn set_decoration(
        &mut self,
        id: u32,
        decoration: spirv::Decoration,
        argument: u32,
    ) -> Result<(), ErrorCode> {
        unsafe {
            check!(sc_internal_compiler_set_decoration(
                self.sc_compiler,
                id,
                decoration.as_raw(),
                argument,
            ));
        }

        Ok(())
    }

    pub fn get_entry_points(&self) -> Result<Vec<spirv::EntryPoint>, ErrorCode> {
        let mut entry_points_raw = ptr::null_mut();
        let mut entry_points_raw_length = 0 as usize;

        unsafe {
            check!(sc_internal_compiler_get_entry_points(
                self.sc_compiler,
                &mut entry_points_raw,
                &mut entry_points_raw_length,
            ));

            let entry_points = (0..entry_points_raw_length)
                .map(|offset| {
                    let entry_point_raw_ptr = entry_points_raw.offset(offset as isize);
                    let entry_point_raw = *entry_point_raw_ptr;
                    let name = match CStr::from_ptr(entry_point_raw.name)
                        .to_owned()
                        .into_string()
                    {
                        Ok(n) => n,
                        _ => return Err(ErrorCode::Unhandled),
                    };

                    let entry_point = spirv::EntryPoint {
                        name,
                        execution_model: try!(spirv::ExecutionModel::from_raw(
                            entry_point_raw.execution_model
                        )),
                        work_group_size: spirv::WorkGroupSize {
                            x: entry_point_raw.work_group_size_x,
                            y: entry_point_raw.work_group_size_y,
                            z: entry_point_raw.work_group_size_z,
                        },
                    };

                    check!(sc_internal_free_pointer(
                        entry_point_raw.name as *mut c_void,
                    ));
                    check!(sc_internal_free_pointer(entry_point_raw_ptr as *mut c_void));

                    Ok(entry_point)
                })
                .collect::<Result<Vec<_>, _>>();

            Ok(try!(entry_points))
        }
    }

    pub fn get_shader_resources(&self) -> Result<spirv::ShaderResources, ErrorCode> {
        unsafe {
            let mut shader_resources_raw = mem::zeroed();
            check!(sc_internal_compiler_get_shader_resources(
                self.sc_compiler,
                &mut shader_resources_raw,
            ));

            let fill_resources = |array_raw: &ScResourceArray| {
                let resources_raw = slice::from_raw_parts(array_raw.data, array_raw.num);
                let resources = resources_raw
                    .iter()
                    .map(|resource_raw| {
                        let name = match CStr::from_ptr(resource_raw.name).to_owned().into_string()
                        {
                            Ok(n) => n,
                            _ => return Err(ErrorCode::Unhandled),
                        };

                        check!(sc_internal_free_pointer(resource_raw.name as *mut c_void,));

                        Ok(spirv::Resource {
                            id: resource_raw.id,
                            type_id: resource_raw.type_id,
                            base_type_id: resource_raw.base_type_id,
                            name,
                        })
                    })
                    .collect::<Result<Vec<_>, ErrorCode>>();

                check!(sc_internal_free_pointer(array_raw.data as *mut c_void,));

                resources
            };

            let uniform_buffers = fill_resources(&shader_resources_raw.uniform_buffers)?;
            let storage_buffers = fill_resources(&shader_resources_raw.storage_buffers)?;
            let stage_inputs = fill_resources(&shader_resources_raw.stage_inputs)?;
            let stage_outputs = fill_resources(&shader_resources_raw.stage_outputs)?;
            let subpass_inputs = fill_resources(&shader_resources_raw.subpass_inputs)?;
            let storage_images = fill_resources(&shader_resources_raw.storage_images)?;
            let sampled_images = fill_resources(&shader_resources_raw.sampled_images)?;
            let atomic_counters = fill_resources(&shader_resources_raw.atomic_counters)?;
            let push_constant_buffers =
                fill_resources(&shader_resources_raw.push_constant_buffers)?;
            let separate_images = fill_resources(&shader_resources_raw.separate_images)?;
            let separate_samplers = fill_resources(&shader_resources_raw.separate_samplers)?;

            Ok(spirv::ShaderResources {
                uniform_buffers,
                storage_buffers,
                stage_inputs,
                stage_outputs,
                subpass_inputs,
                storage_images,
                sampled_images,
                atomic_counters,
                push_constant_buffers,
                separate_images,
                separate_samplers,
            })
        }
    }
}

impl Drop for Compiler {
    fn drop(&mut self) {
        unsafe {
            sc_internal_compiler_delete(self.sc_compiler);
        }
    }
}
