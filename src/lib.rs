use geo::{ConvexHull, HaversineDestination, MapCoords};

trait SunPosition {
    fn sun_position(&self, unixtime_ms: i64) -> sun::Position;
}

impl SunPosition for geo::Coordinate {
    fn sun_position(&self, unixtime_ms: i64) -> sun::Position {
        sun::pos(unixtime_ms, self.y, self.x)
    }
}

impl SunPosition for geo::Point {
    fn sun_position(&self, unixtime_ms: i64) -> sun::Position {
        sun::pos(unixtime_ms, self.y(), self.x())
    }
}

trait Shadow {
    /// `height`: in meters
    fn shadow(&self, height: f64, unixtime_ms: i64) -> geo::Polygon;
}

impl Shadow for geo::Rect {
    fn shadow(&self, height: f64, unixtime_ms: i64) -> geo::Polygon {
        self.to_polygon().shadow(height, unixtime_ms)
    }
}

impl Shadow for geo::Polygon {
    fn shadow(&self, height: f64, unixtime_ms: i64) -> geo::Polygon {
        let shadow_extent = self.map_coords(|coord| {
            let sun_position = coord.sun_position(unixtime_ms);
            let shadow_length = shadow_length(&sun_position, height);
            geo::Point(coord)
                .haversine_destination(sun_position.azimuth, shadow_length)
                .0
        });

        geo::MultiPolygon::new(vec![
            self.clone(), // TODO: Remove this clone
            shadow_extent
        ]).convex_hull()
    }
}

fn shadow_length(sun_position: &sun::Position, height: f64) -> f64 {
    height / sun_position.altitude.tan()
}

#[test]
fn test_shadow_length_nyc_winter_solstice_2017() {
    let point = geo::point! { x: -74.005941, y: 40.712784 };
    let sun_position = point.sun_position(1_419_184_800_000);

    let length = shadow_length(&sun_position, 50.);

    assert_eq!(24., sun_position.altitude.to_degrees().round());
    assert_eq!(196., sun_position.azimuth.to_degrees().round());
    assert_eq!(112., length.round());
}

#[test]
fn test_shadow_length_nyc_summer_solstice_2017() {
    let point = geo::point! { x: -74.005941, y: 40.712784 };
    let sun_position = point.sun_position(1_655_830_800_000);

    let length = shadow_length(&sun_position, 50.);

    assert_eq!(73., sun_position.altitude.to_degrees().round());
    assert_eq!(181., sun_position.azimuth.to_degrees().round());
    assert_eq!(16., length.round());
}
