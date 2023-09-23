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
