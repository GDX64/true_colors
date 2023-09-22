use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run_image(img: Vec<u8>) -> Vec<usize> {
    let hsls: Vec<_> = img
        .chunks_exact(4)
        .map(|chunk| {
            RGBA {
                r: chunk[0],
                g: chunk[1],
                b: chunk[2],
                a: chunk[3],
            }
            .to_hsl()
        })
        .collect();

    let histogram = HSL::histogram(hsls);
    histogram
}

struct RGBA {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

struct HSL {
    h: f64,
    s: f64,
    l: f64,
}

impl HSL {
    fn bucket(&self) -> usize {
        self.h.floor() as usize
    }

    fn histogram(HSLs: Vec<HSL>) -> Vec<usize> {
        const N: usize = 360;
        let mut histogram = vec![0; N];
        HSLs.iter().for_each(|x| {
            histogram[x.bucket() % N] += 1;
        });
        histogram
    }
}

impl RGBA {
    fn to_hsl(&self) -> HSL {
        let r = self.r as f64 / 255.0;
        let g = self.g as f64 / 255.0;
        let b = self.b as f64 / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let mut h = (max + min) / 2.0;
        let mut s = h;
        let l = h;
        if max == min {
            h = 0.0;
            s = 0.0;
        } else {
            let d = max - min;
            s = if l > 0.5 {
                d / (2.0 - max - min)
            } else {
                d / (max + min)
            };
            if max == r {
                h = (g - b) / d + if g < b { 6.0 } else { 0.0 };
            } else if max == g {
                h = (b - r) / d + 2.0;
            } else if max == b {
                h = (r - g) / d + 4.0;
            }
            h /= 6.0;
        }
        HSL {
            h: h * 360.0,
            s: s * 100.0,
            l: l * 100.0,
        }
    }
}

#[derive(Debug)]
enum QuadNode<T> {
    Empty,
    Leaf(T),
    Branch(Box<[QuadTree<T>; 4]>),
}

#[derive(Debug)]
struct QuadTree<T> {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    root: QuadNode<T>,
}

trait CanBeLeaf {
    fn x(&self) -> f64;
    fn y(&self) -> f64;
}

impl<T: CanBeLeaf> QuadTree<T> {
    fn new(x: f64, y: f64, height: f64, width: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            root: QuadNode::<T>::Empty,
        }
    }

    fn insert(&mut self, value: T) -> Option<T> {
        if self.x > value.x()
            || self.y > value.y()
            || self.x + self.width < value.x()
            || self.y + self.height < value.y()
        {
            return Some(value);
        }
        match self.root {
            QuadNode::Empty => {
                self.root = QuadNode::Leaf(value);
            }
            QuadNode::Leaf(_) => {
                let mut nodes = [
                    QuadTree::new(self.x, self.y, self.width / 2.0, self.height / 2.0),
                    QuadTree::new(
                        self.x + self.width / 2.0,
                        self.y,
                        self.width / 2.0,
                        self.height / 2.0,
                    ),
                    QuadTree::new(
                        self.x,
                        self.y + self.height / 2.0,
                        self.width / 2.0,
                        self.height / 2.0,
                    ),
                    QuadTree::new(
                        self.x + self.width / 2.0,
                        self.y + self.height / 2.0,
                        self.width / 2.0,
                        self.height / 2.0,
                    ),
                ];
                Self::try_insert_on_nodes(&mut nodes, value);
                self.root = QuadNode::Branch(Box::new(nodes));
            }
            QuadNode::Branch(ref mut nodes) => {
                Self::try_insert_on_nodes(nodes, value);
            }
        };
        None
    }

    fn try_insert_on_nodes(nodes: &mut [QuadTree<T>; 4], value: T) {
        nodes[0]
            .insert(value)
            .and_then(|v| nodes[1].insert(v))
            .and_then(|v| nodes[2].insert(v))
            .and_then(|v| nodes[3].insert(v));
    }
}

#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
}

impl CanBeLeaf for Point {
    fn x(&self) -> f64 {
        self.x
    }

    fn y(&self) -> f64 {
        self.y
    }
}

mod test {
    #[test]
    fn test_quad() {
        use super::*;
        let mut tree = QuadTree::new(0.0, 0.0, 100.0, 100.0);
        tree.insert(Point { x: 10.0, y: 10.0 });
        tree.insert(Point { x: 10.0, y: 60.0 });
        println!("{tree:?}")
    }
}
