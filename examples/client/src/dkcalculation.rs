use std::f64::consts::PI;

pub struct DKPoint{
  pub x:f64,
  pub y:f64,
}

// #[derive(Default)]
pub struct DKCalculation {  
  pub sensor_width: f64,  
  pub lens_focal_length: f64,  
  pub angle_of_view: f64,
  pub distance: f64,
  pub now_pan: f64,
  pub now_tilt: f64,
}

impl Default for DKCalculation {
  fn default() -> Self {
    let sensor_width = (std::env::var("VIEW_ANGLE_W").unwrap().parse::<f64>().unwrap() / 2.0 * PI).tan() * std::env::var("LENS_DISTANCE").unwrap().parse::<f64>().unwrap() * 2.0;
    let lens_focal_length = std::env::var("LENS_DISTANCE").unwrap().parse::<f64>().unwrap() * 1.0;
    let angle_of_view = (sensor_width / 2.0 / lens_focal_length).atan() * 2.0 * 180.0 / PI;
    let distance = std::env::var("CAMERA_SCREEN_WIDTH").unwrap().parse::<f64>().unwrap() / (angle_of_view / 2.0 / 180.0 * PI).tan();
    let now_tilt = (std::env::var("INSTALLATION_HEIGHT").unwrap().parse::<f64>().unwrap() / std::env::var("SCREEN_CENTER_DISTANCE").unwrap().parse::<f64>().unwrap()).atan() * (180.0 / PI);
    Self { 
      sensor_width,
      lens_focal_length,
      angle_of_view,
      distance,
      now_pan:90.0,
      now_tilt,
    }
  }
}
 
impl DKCalculation  {    
  // 속도 구하기
  pub fn distance_speed(distance:f64, time:f64) -> f64 {
    (distance/(time/1000.0)*3600.0)/1000.0
  }

  // 거리 구하기
  pub fn point_to_point_distance(p1:DKPoint, p2:DKPoint) -> f64 {
    ((p2.x - p1.x).powf(2.0)+(p2.y - p1.y).powf(2.0)).sqrt()
  }
}
