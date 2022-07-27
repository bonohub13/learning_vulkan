pub struct ValidationLayer<'a> {
    pub required_validation_layers: &'a [&'a str],
    pub is_enable: bool,
}

pub fn check_validation_layer_support(entry: &ash::Entry) -> bool {
    use crate::{constants::VK_VALIDATION_LAYERS, tools as vk_tools};

    let layer_properties = entry
        .enumerate_instance_layer_properties()
        .expect("Failed to enumerate Instance Layers Properties");

    if layer_properties.len() <= 0 {
        eprintln!("No available layers");

        false
    } else {
        println!("Instance Available Layers:");
        for layer_property in layer_properties.iter() {
            let layer_name = vk_tools::raw_charptr_to_string(&layer_property.layer_name);

            println!("\t{}", layer_name);
        }

        for required_layer_name in (*VK_VALIDATION_LAYERS.required_validation_layers).iter() {
            let mut is_layer_found = false;

            for layer_property in layer_properties.iter() {
                let test_layer_name = vk_tools::raw_charptr_to_string(&layer_property.layer_name);

                if (*required_layer_name) == test_layer_name {
                    is_layer_found = true;
                    break;
                }
            }

            if !is_layer_found {
                return is_layer_found;
            }
        }

        true
    }
}
