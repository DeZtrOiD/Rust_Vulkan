
use ash::{Device, vk};
pub struct VulkanSampler {
    pub sampler: vk::Sampler,
    pub device: Device
}

impl Drop for VulkanSampler {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_sampler(self.sampler, None);
        }
    }
}

pub struct VulkanSamplerBuilder<'a> {
    pub info: vk::SamplerCreateInfo<'a>,
    pub device: &'a Device
}

impl<'a> VulkanSamplerBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            info: vk::SamplerCreateInfo{
                mag_filter: vk::Filter::LINEAR,  // Фильтрация при увеличении текстуры.
                min_filter: vk::Filter::LINEAR,  // Фильтрация при уменьшении текстуры.
                address_mode_u: vk::SamplerAddressMode::REPEAT,  // Поведение при выходе за пределы текстуры по оси U.
                address_mode_v: vk::SamplerAddressMode::REPEAT,  // Поведение при выходе за пределы текстуры по оси V.
                address_mode_w: vk::SamplerAddressMode::REPEAT,  // Поведение при выходе за пределы текстуры по оси W (для 3д текстур).            
                mipmap_mode: vk::SamplerMipmapMode::NEAREST,
                anisotropy_enable: vk::TRUE,
                max_anisotropy: 16.0,
                border_color: vk::BorderColor::INT_OPAQUE_BLACK,
                unnormalized_coordinates: vk::FALSE,
                compare_enable: vk::FALSE,
                compare_op: vk::CompareOp::ALWAYS,
                min_lod: 0.0,
                max_lod: vk::LOD_CLAMP_NONE,
                ..Default::default()
            },
            device: device,
        }
    }
    // какие-то очень тонкие флаги для субсемплеров
    pub fn flags(mut self, flags: vk::SamplerCreateFlags) -> Self {
        self.info.flags = flags;
        self
    }
    /// Устанавливает фильтр, применяемый при увеличении текстуры (магнификация).
    pub fn mag_filter(mut self, mag_filter: vk::Filter) -> Self {
        self.info.mag_filter = mag_filter;
        self
    }
    /// Устанавливает фильтр, применяемый при уменьшении текстуры (минификация).
    pub fn min_filter(mut self, min_filter: vk::Filter) -> Self {
        self.info.min_filter = min_filter;
        self
    }
    /// Устанавливает режим фильтрации между уровнями mip-уровней текстуры.
    pub fn mipmap_mode(mut self, mipmap_mode: vk::SamplerMipmapMode) -> Self {
        self.info.mipmap_mode = mipmap_mode;
        self
    }
    /// Устанавливает режим адресации для координаты U (горизонтальная ось текстуры).
    /// Определяет, что делать при выходе координаты за пределы [0, 1].
    /// Варианты: `REPEAT`, `MIRRORED_REPEAT`, `CLAMP_TO_EDGE`, и т.д.
    pub fn address_mode_u(mut self, address_mode_u: vk::SamplerAddressMode) -> Self {
        self.info.address_mode_u = address_mode_u;
        self
    }
    /// Устанавливает режим адресации для координаты V (вертикальная ось текстуры).
    pub fn address_mode_v(mut self, address_mode_v: vk::SamplerAddressMode) -> Self {
        self.info.address_mode_v = address_mode_v;
        self
    }
    /// Устанавливает режим адресации для координаты W (используется в 3D-текстурах).
    pub fn address_mode_w(mut self, address_mode_w: vk::SamplerAddressMode) -> Self {
        self.info.address_mode_w = address_mode_w;
        self
    }
    /// Смещение уровня детализации (LOD — Level of Detail) при выборе mip-уровня.
    /// Полезно для управления размытием или резкостью текстуры.
    pub fn mip_lod_bias(mut self, mip_lod_bias: f32) -> Self {
        self.info.mip_lod_bias = mip_lod_bias;
        self
    }
    /// Включает/выключает анизотропную фильтрацию.
    /// Улучшает качество текстур при просмотре под углом.
    /// Требует поддержки соответствующего флага в физическом устройстве.
    pub fn anisotropy_enable(mut self, anisotropy_enable: vk::Bool32) -> Self {
        self.info.anisotropy_enable = anisotropy_enable;
        self
    }
    /// Устанавливает максимальный уровень анизотропии (например, 4.0, 8.0, 16.0).
    /// Чем выше — тем лучше качество при наклонном взгляде, но дороже с точки зрения производительности.
    pub fn max_anisotropy(mut self, max_anisotropy: f32) -> Self {
        self.info.max_anisotropy = max_anisotropy;
        self
    }
    /// Включает режим сравнения, используемый в основном для shadow mapping.
    /// Если включён — сэмплер сравнивает выборку с заданным значением глубины.
    pub fn compare_enable(mut self, compare_enable: vk::Bool32) -> Self {
        self.info.compare_enable = compare_enable;
        self
    }
    /// Оператор сравнения, используемый при `compare_enable = VK_TRUE`.
    /// Например: `LESS`, `GREATER`, `EQUAL`, и т.д.
    pub fn compare_op(mut self, compare_op: vk::CompareOp) -> Self {
        self.info.compare_op = compare_op;
        self
    }
    /// Минимальный уровень детализации (LOD), который может быть использован.
    /// LOD = 0 соответствует базовому разрешению текстуры.
    pub fn min_lod(mut self, min_lod: f32) -> Self {
        self.info.min_lod = min_lod;
        self
    }
    /// Максимальный уровень детализации (LOD), который может быть использован.
    /// Например, если текстура имеет 5 mip-уровней, `max_lod = 4.0`.
    pub fn max_lod(mut self, max_lod: f32) -> Self {
        self.info.max_lod = max_lod;
        self
    }
    /// Цвет границы, используемый при режиме адресации `CLAMP_TO_BORDER`.
    /// Возможные значения: чёрный, белый, прозрачный и т.д.
    pub fn border_color(mut self, border_color: vk::BorderColor) -> Self {
        self.info.border_color = border_color;
        self
    }
    /// Если `true`, координаты текстуры интерпретируются как абсолютные пиксельные координаты,
    /// а не нормализованные [0, 1]. Редко используется.
    pub fn unnormalized_coordinates(mut self, unnormalized_coordinates: vk::Bool32) -> Self {
        self.info.unnormalized_coordinates = unnormalized_coordinates;
        self
    }

    pub fn build(&self) -> Result<VulkanSampler, &'static str> {
        let samp = unsafe {
            self.device.create_sampler(&self.info, None).map_err(|_| "Err create_sampler")?
        };
        Ok(VulkanSampler{
            sampler: samp,
            device: self.device.clone(),
        })
    }
}

