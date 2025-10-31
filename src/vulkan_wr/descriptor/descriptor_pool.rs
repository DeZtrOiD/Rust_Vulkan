// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: descriptor pool wrapper
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{vk, Device};
use super::descriptor_set::{VulkanDescriptorSet};
use super::descriptor_set_layout::{VulkanDescriptorSetLayout};

#[derive(Clone)]
pub struct VulaknDescriptorPool {
    pub _pool: vk::DescriptorPool,
    pub _device: Device,
}

type DResult<T> = Result<T, &'static str>;

impl VulaknDescriptorPool {
    /// # Аргументы
    /// * `device` — ссылка на Vulkan logical device
    /// * `pool_size_vec` — вектор указывающий, сколько и каких дескрипторов должно быть в пуле
    /// * `max_sets` — максимальное количество descriptor sets, которое можно выделить из пула
    /// * `flags` — флаги управление памятью, можно разрешить reset (FREE_DESCRIPTOR_SET)
    pub fn try_new(
        device: &Device,
        pool_size_vec: &Vec<vk::DescriptorPoolSize>,
        max_sets: u32,
        flags: Option<vk::DescriptorPoolCreateFlags>
    ) -> DResult<Self> {

        let flags_unwrap = flags.or(Some(vk::DescriptorPoolCreateFlags::empty())).unwrap();
        let create_info = vk::DescriptorPoolCreateInfo {
            pool_size_count: pool_size_vec.len() as u32,
            p_pool_sizes: pool_size_vec.as_ptr(),
            max_sets: max_sets,
            flags: flags_unwrap,
            ..Default::default()
        };

        let pool = unsafe { device.create_descriptor_pool(&create_info, None).map_err(|_| "Err create_descriptor_pool")? };
        Ok(Self { _pool: (pool), _device: (device.clone()) })
    }

    // flags зарезервированны
    pub fn reset(&self) -> DResult<()> {
        unsafe {
            self._device.reset_descriptor_pool(self._pool, vk::DescriptorPoolResetFlags::empty())
                .map_err(|_| "Failed to reset descriptor pool")
        }
    }

    /// # Аргументы
    /// * `layouts` — срез обёрток VulkanDescriptorSetLayout, определяющих структуру каждого descriptor set
    pub fn allocate_descriptor_sets(
        &self, 
        layouts: &[VulkanDescriptorSetLayout]
    ) -> DResult<Vec<VulkanDescriptorSet>> {
        let vec_layouts: Vec<vk::DescriptorSetLayout> = layouts.iter().map(|l| l.layout).collect();
        let allocate_info = vk::DescriptorSetAllocateInfo {
            descriptor_pool: self._pool,
            descriptor_set_count: vec_layouts.len() as u32,
            p_set_layouts: vec_layouts.as_ptr(),
            ..Default::default()
        };

        let sets = unsafe {
            self._device.allocate_descriptor_sets(&allocate_info)
                .map_err(|_| "Failed to allocate multiple descriptor sets")?
        };

        Ok(sets.into_iter().map(|set| VulkanDescriptorSet {
            set,
            _device: self._device.clone(),
        }).collect())
    }

    pub fn free_descriptor_sets(&self, descriptor_sets: Vec<VulkanDescriptorSet>) -> DResult<()> {
        let sets: Vec<vk::DescriptorSet> = descriptor_sets.into_iter().map(|ds| ds.set).collect();
        unsafe {
            self._device.free_descriptor_sets(self._pool, &sets)
                .map_err(|_| "Failed to free descriptor sets")
        }
    }

    /// Once allocated, descriptor sets can be updated with a combination of write and copy operations
    /// # Аргументы
    /// * `descriptor_writes` — массив vk::WriteDescriptorSet для записи новых данных
    /// * `descriptor_copies` — массив vk::CopyDescriptorSet для копирования существующих дескрипторов
    pub fn update_descriptor_sets<>(
        &self,
        descriptor_writes: &[vk::WriteDescriptorSet],
        descriptor_copies: &[vk::CopyDescriptorSet],
    ) {
        unsafe {
            self._device.update_descriptor_sets(descriptor_writes, descriptor_copies);
        }
    }
}

impl Drop for VulaknDescriptorPool {
    fn drop(&mut self) {
        unsafe { self._device.destroy_descriptor_pool(self._pool, None) };
    }
}