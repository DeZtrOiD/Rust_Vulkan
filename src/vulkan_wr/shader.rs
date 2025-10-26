// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc:
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{vk, Device};
use std::io::Read;

pub(super) struct VulkanShader {
    pub _shader: vk::ShaderModule,
    _device:Device,
}

pub type SResult<T> = Result<T, &'static str>;


impl VulkanShader {
    pub(super) fn try_new(device: &Device, path: &str) -> SResult<Self> {
        let mut file = std::fs::File::open(path).map_err(|_| "Unable to load shader!")?;
        let mut raw = vec![];
        let file_size = file.read_to_end(&mut raw).map_err(|_| "")?;

        // вулкан хочет u32, а не u8 круто
        let mut dwords = vec![];
        for chunk in raw.chunks_exact(4) {
            dwords.push(u32::from_le_bytes([ chunk[0], chunk[1], chunk[2], chunk[3] ]));
        }

        let shader_info = vk::ShaderModuleCreateInfo {
            code_size: file_size,
            p_code: dwords.as_ptr(),
            ..Default::default()
        };

        let shader = unsafe{ device.create_shader_module(&shader_info, None).map_err(|_| "Err create_shader_module")?};

        Ok(Self{
            _shader: shader,
            _device: device.clone(),
        })
    }

    pub fn destroy(&self) {
        unsafe { self._device.destroy_shader_module(self._shader, None) };
    }

}