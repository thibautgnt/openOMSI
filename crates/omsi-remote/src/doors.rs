use omsi_sim::VehicleType;

fn leaf(var: &str) -> Option<usize> {
    var.trim().to_ascii_lowercase().strip_prefix("door_")?.parse().ok()
}

fn median(mut v: Vec<f32>) -> Option<f32> {
    v.sort_by(f32::total_cmp);
    v.get(v.len() / 2).copied()
}

pub fn doorways(ty: &VehicleType) -> Vec<Vec<usize>> {
    let mut hinges: Vec<(usize, Vec<f32>, Vec<f32>)> = Vec::new();
    for (i, vm) in ty.meshes.iter().enumerate() {
        let Some(def) = ty.model.meshes.get(vm.def_index) else { continue };
        let Some((n, anim)) = def.animations.iter().find_map(|a| Some((leaf(&a.variable)?, a))) else { continue };
        let at = match hinges.iter().position(|e| e.0 == n) {
            Some(k) => k,
            None => {
                hinges.push((n, Vec::new(), Vec::new()));
                hinges.len() - 1
            }
        };
        let hinge = anim.origins.iter().find_map(|o| match o {
            omsi_model::AnimOrigin::Trans(t) => Some(t[1]),
            _ => None,
        });
        match hinge {
            Some(y) => hinges[at].1.push(y),
            None => {
                if let Some(d) = ty.mesh_data(i).filter(|d| !d.positions.is_empty()) {
                    hinges[at].2.push(d.positions.iter().map(|p| vm.pivot.transform_point3(*p).y).sum::<f32>() / d.positions.len() as f32);
                }
            }
        }
    }
    let mut leaves: Vec<(usize, f32)> = hinges.into_iter().filter_map(|(n, h, c)| Some((n, if h.is_empty() { median(c)? } else { median(h)? }))).collect();
    leaves.sort_by(|a, b| b.1.total_cmp(&a.1));
    let mut ways: Vec<Vec<(usize, f32)>> = Vec::new();
    for l in leaves {
        match ways.last_mut() {
            Some(w) if w.last().is_some_and(|p| (p.1 - l.1).abs() < 1.6) => w.push(l),
            _ => ways.push(vec![l]),
        }
    }
    ways.into_iter().map(|w| w.into_iter().map(|(n, _)| n).collect()).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_leaf_is_a_numbered_door_variable() {
        assert_eq!(super::leaf("door_3"), Some(3));
        assert_eq!(super::leaf("Door_0"), Some(0));
        assert_eq!(super::leaf("door_2L"), None);
        assert_eq!(super::leaf("doorlight"), None);
    }
}
