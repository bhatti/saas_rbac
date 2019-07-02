//#![crate_name = "doc"]


////////////////////////////////////////////////////////////////////////////////
/// Defines helper method to calculate distance between two latitude/longitude
///
pub fn distance_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let earth_radius_km: f64 = 6371.0;
    let deg_lat = to_radians(lat2-lat1);
    let deg_lon = to_radians(lon2-lon1);
    let lat1 = to_radians(lat1);
    let lat2 = to_radians(lat2);

    let a = (deg_lat/2.0).sin() * (deg_lat/2.0).sin() + (deg_lon/2.0).sin() * (deg_lon/2.0).sin() * lat1.cos() * lat2.cos();
    let c = 2.0 * a.sqrt().atan2((1.0-a).sqrt());
    earth_radius_km * c
}

fn to_radians(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}


#[cfg(test)]
mod tests {
    use plexrbac::utils::distance::distance_km;

    #[test]
    fn test_geo() {
        let lat1 = 47.620422;
        let lon1 = -122.349358;
        let lat2 = 46.879967;
        let lon2 = -121.726906;
        assert!(distance_km(lat1, lon1, lat2, lon2) < 100.0);
    }
}
