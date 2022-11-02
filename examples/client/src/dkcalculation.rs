use std::f64::consts::PI;

pub struct DKAngle {
  pub tilt_angle:f64,
  pub pan_angle:f64,
  pub zoom:f64,
}

#[derive(Copy, Clone)]
pub struct DKPoint {
  pub x:f64,
  pub y:f64,
}

pub struct DKPolygon {
  pub point:Vec<DKPoint>,
}

#[derive(Copy, Clone)]
pub struct DKLine {
  pub p1:DKPoint,
  pub p2:DKPoint,
}

pub struct DKScreen {
  pub width:f64,
  pub height:f64,
}

pub struct DKBox {
  pub top:f64,
  pub bottom:f64,
  pub left:f64,
  pub right:f64,
}

pub struct DKAnswer {
  pub direction:bool,
  pub cross:bool,
}

pub struct DKFcltInstallInfo {
  pub view_angle_w: f64,
  pub lens_distance: f64,
  pub camera_screen_width: f64,
  pub installation_height: f64,
}

// #[derive(Default)]
pub struct DKCalculation {  
  pub camera_screen_width: f64,
  pub camera_screen_height: f64,  
  pub sensor_width: f64,  
  pub lens_focal_length: f64,  
  pub angle_of_view: f64,
  pub distance: f64,
  pub now_pan: f64,
  pub now_tilt: f64,
  pub dk_fclt_install_info: DKFcltInstallInfo,
}

impl DKCalculation  {    
  // constructor
  pub fn new(fclt_install_info:DKFcltInstallInfo, screen_width:f64, screen_height:f64) -> Self {
    let sensor_width =  (fclt_install_info.view_angle_w / 2.0 * PI).tan() * fclt_install_info.lens_distance * 2.0;
    let lens_focal_length = fclt_install_info.lens_distance * 1.0;
    let angle_of_view = (sensor_width / 2.0 / lens_focal_length).atan() * 2.0 * 180.0 / PI;
    let distance = fclt_install_info.camera_screen_width / (angle_of_view / 2.0 / 180.0 * PI).tan();
    let now_tilt = ( fclt_install_info.installation_height / fclt_install_info.camera_screen_width).atan() * (180.0 / PI);
    let dk_fclt_install_info = fclt_install_info;
    let camera_screen_width = screen_width;
    let camera_screen_height = screen_height;
    Self { 
      camera_screen_width,
      camera_screen_height,
      sensor_width,
      lens_focal_length,
      angle_of_view,
      distance,
      now_pan:90.0,
      now_tilt,
      dk_fclt_install_info
    }
  }

  // 속도 측정
  pub fn speed_measurement(&self, box_a:DKBox, box_b:DKBox, time:f64) -> f64 {
    let box1 = DKCalculation::coordinate_leveling(self, box_a);
    let box2 = DKCalculation::coordinate_leveling(self, box_b);
    //카메라에서 각도 구하기
    let box1_angle = DKCalculation::point_zoom(self, box1);
    let box2_angle = DKCalculation::point_zoom(self, box2);
    //지면 좌표 구하기
    let box1_coordinate = DKCalculation::ground_coordinate(self, box1_angle);
    let box2_coordinate = DKCalculation::ground_coordinate(self, box2_angle);
    //거리 구하기
    let distance = DKCalculation::point_to_point_distance(box1_coordinate, box2_coordinate);
    //속도 구하기
    let speed = DKCalculation::distance_speed(distance,time);

    speed
  }

  // 속도 구하기
  pub fn distance_speed(distance:f64, time:f64) -> f64 {
    (distance/(time/1000.0)*3600.0)/1000.0
  }

  // 거리 구하기
  pub fn point_to_point_distance(p1:DKPoint, p2:DKPoint) -> f64 {
    ((p2.x - p1.x).powf(2.0)+(p2.y - p1.y).powf(2.0)).sqrt()
  }

  // 지면 좌표 구하기
  pub fn ground_coordinate(&self, angle:DKAngle) -> DKPoint {
    let tmp_y = self.dk_fclt_install_info.installation_height / (angle.tilt_angle * PI).tan();
    let y = (angle.pan_angle * PI / 180.0).cos() * tmp_y;
    let x = (angle.pan_angle * PI / 180.0).sin() * tmp_y;
    let point = DKPoint {
      x:x,
      y:y,
    };
    
    point
  }

  pub fn coordinate_leveling(&self, input_box :DKBox) -> DKBox {
    let return_box = DKBox {
      top : self.camera_screen_height - input_box.top,
      left : input_box.left,
      right : input_box.right,
      bottom : self.camera_screen_height - input_box.bottom
    };

    return_box
  }

  pub fn point_zoom (&self, input_box :DKBox) -> DKAngle {
    let screen_width = self.camera_screen_width;
    let screen_height = self.camera_screen_height;
    let basic_width = self.camera_screen_width;
    let basic_height = self.camera_screen_height;
    let now_zoom = 1.0;
    let now_pan = self.now_pan;
    let now_tilt = self.now_tilt;
    let distance = self.distance;

    let coord_x = ((((input_box.right - input_box.left) / 2.0) + input_box.left) / screen_width) * basic_width;
    let coord_y = ((((input_box.top - input_box.bottom) / 2.0) + input_box.left) / screen_width) * basic_width;

    let tmp_y = (basic_height / 2.0) - coord_y; 
    let tmp_x = (basic_width / 2.0) - coord_x; 
    
    //------------- zoom 레벨에 따른 pan offset 구하기	
    // 거리 비율에 따른 회전각 산정
    let stand_tilt = now_tilt + ((tmp_y / distance).atan() * (180.0 / PI));
    let stand_pan_distance = (stand_tilt * PI / 180.0).cos() * distance;
    let tmp_pan = (tmp_x / stand_pan_distance).atan() * (180.0 / PI);

    //최종각 panAngle 설정
    let mut pan_angle = now_pan + tmp_pan;

    //------------- zoom 레벨에 따른 tilt offset 구하기
    //거리 비율에 따른 회전각 산정
    let stand_distance = distance / (stand_tilt * PI / 180.0).tan();
    let object_distance = stand_distance / (tmp_pan * PI / 180.0).cos();
    let tmp_tilt = (distance/object_distance).atan() * (180.0 / PI);

    let mut tilt_angle = tmp_tilt;

    if now_tilt > 45.0 && tilt_angle < 0.0 {
      tilt_angle = tilt_angle * -1.0;
      pan_angle = pan_angle - 180.0;
    }
    if pan_angle < 0.0 {
      pan_angle = pan_angle + 360.0;
    }

    let return_angle = DKAngle{
      pan_angle,
      tilt_angle,
      zoom: now_zoom,
    };

    return_angle    
  }

  //좌상단 좌표계를 우하단 좌표계로변경.
  pub fn coordinate_adjustment(&self, point:DKPoint) -> DKPoint {
    let dy = self.camera_screen_height - point.y;
    let dx = point.x;
    let return_point = DKPoint {
      x: dx,
      y: dy,
    };

    return_point
  }

  //스크린 차이를 동일 스크린 사이즈로 변경
  pub fn screen_size_adjustment(&self, screen:DKScreen, point:DKPoint) -> DKPoint {
    let width_ratio = self.camera_screen_width / screen.width;
    let height_ratio = self.camera_screen_height / screen.height;
    let return_point= DKPoint {
      x : point.x * width_ratio,
      y : point.y * height_ratio,
    };

    return_point
  }

  pub fn degree_cal(p1:DKPoint, p2:DKPoint) -> f64 {
    let x = p2.x - p1.x;
    let y = p2.y - p1.y;
    let radian = f64::atan2(y, x);
    let degree = radian * 180.0 / PI;

    degree
  }

  //두 선의 교점
  pub fn intersection_cal(line1:DKLine, line2:DKLine) -> DKPoint {
    let x1 = line1.p1.x;
    let y1 = line1.p1.y;
    let x2 = line1.p2.x;
    let y2 = line1.p2.y;
    let x3 = line2.p1.x;
    let y3 = line2.p1.y;
    let x4 = line2.p2.x;
    let y4 = line2.p2.y;
    
    let x:f64;
    let y:f64;

    if ((x1-x2)*(y3-y4)-(y1-y2)*(x3-x4)) == 0.0 {
      x = 0.0;
      y = 0.0;
    }
    else {
      x = ((x1*y2-y1*x2)*(x3-x4)-(x1-x2)*(x3*y4-y3*x4)) / ((x1-x2)*(y3-y4)-(y1-y2)*(x3-x4));
      y = ((x1*y2-y1*x2)*(y3-y4)-(y1-y2)*(x3*y4-y3*x4)) / ((x1-x2)*(y3-y4)-(y1-y2)*(x3-x4));
    }

    let return_point= DKPoint {
      x, 
      y,
    };

    return_point
  }

  pub fn p_top_distance(p1:DKPoint, p2:DKPoint) -> f64 {
    let f = (f64::powf(p2.x-p1.x, 2.0) + f64::powf(p2.y-p1.y, 2.0)).sqrt();

    f
  }

  pub fn collision_cal(&self, line:DKLine, point:DKPoint) -> bool {
    let result:bool;
    let point1_to_point_dis = DKCalculation::p_top_distance(line.p1, point);
    let point2_to_point_dis = DKCalculation::p_top_distance(line.p2, point);
    let line_dis = DKCalculation::p_top_distance(line.p1, line.p2);
    if point1_to_point_dis + point2_to_point_dis <= line_dis {
      result = true;
    }
    else {
      result = false;
    }

    result
  }

  pub fn line_cross_cal(&self, line1:DKLine, line2:DKLine) -> DKAnswer {
    let mut answer = DKAnswer {
      cross : false,
      direction : false,
    };
    let intersection_point = DKCalculation::intersection_cal(line1, line2);
    if intersection_point.x != 0.0 && intersection_point.y != 0.0 {
      let line1_in_point = DKCalculation::collision_cal(self, line1, intersection_point);
      let line2_in_point = DKCalculation::collision_cal(self, line2, intersection_point);

      if line1_in_point && line2_in_point {
        answer.cross = true;
        let line1_degree = DKCalculation::degree_cal(line1.p1, line1.p2);
        let intersection_degree = DKCalculation::degree_cal(line2.p1, intersection_point);
        let mut angle_difference = intersection_degree - line1_degree;
        if angle_difference < 0.0 && angle_difference < - 180.0 {
          angle_difference = angle_difference + 360.0;
        }
        else if angle_difference > 0.0 && angle_difference > 180.0 {
          angle_difference = angle_difference - 360.0;
        }
        if angle_difference > 0.0 {
          answer.direction = true;
        }
        else if angle_difference < 0.0 {
          answer.direction = false;
        }
        else {
          answer.cross = false;
        }
      }
      else {
        answer.cross = false;
      }
    }
    else {
      answer.cross = false;
    }
    
    answer
  }
  
  pub fn enter_polygon_check(apex_data:DKPolygon, point:DKPoint) -> bool {
    let mut result:bool = false;
    let mut cross_count = 0;    
    for i in 0..apex_data.point.len() {
      let mut j = i+1;
      if j == apex_data.point.len() {
        j = 0
      }
      if (apex_data.point[i].y > point.y) != (apex_data.point[j].y > point.y) {
        let cross_x_point = (apex_data.point[j].x - apex_data.point[i].x) * (point.y - apex_data.point[i].y) / apex_data.point[i].x;
        if point.x < cross_x_point {
          cross_count += 1;
        }
      }
    }
    if 0 == (cross_count % 2) {
      result = false;
    }
    else {
      result = true;
    }

    result
  }
}
