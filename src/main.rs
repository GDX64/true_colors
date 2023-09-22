
pub fn main(){
  
}

struct RGBA{
  r: u8,
  g: u8,
  b: u8,
  a: u8
}

struct HSL{
  h: f64,
  s: f64,
  l: f64
}

impl HSL{
  fn bucket(&self)->usize{
    self.h as usize / 10
  }

  fn histogram(HSLs: Vec<HSL>) -> Vec<usize>{
    const N: usize = 360;
    let mut histogram = vec![0; N];
    HSLs.iter().for_each(|x| {
      histogram[x.bucket()/10] += 1;
    });
    histogram
  }
}

impl RGBA{
  fn to_hsl(&self) -> HSL{
    let r = self.r as f64 / 255.0;
    let g = self.g as f64 / 255.0;
    let b = self.b as f64 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let mut h = (max + min) / 2.0;
    let mut s = h;
    let l = h;
    if max == min{
      h = 0.0;
      s = 0.0;
    }else{
      let d = max - min;
      s = if l > 0.5 {d / (2.0 - max - min)} else {d / (max + min)};
      if max == r{
        h = (g - b) / d + if g < b {6.0} else {0.0};
      }else if max == g{
        h = (b - r) / d + 2.0;
      }else if max == b{
        h = (r - g) / d + 4.0;
      }
      h /= 6.0;
    }
    HSL{
      h: h * 360.0,
      s: s * 100.0,
      l: l * 100.0
    }
  }
}