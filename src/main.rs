use std::cmp;
use std::thread;

extern crate simple;
use simple::{Event, Point, Window};

#[derive(Debug, Copy, Clone)]
struct VirtualPoint(f64, f64, (u8, u8, u8)); // (x,y,color) as doubles

#[derive(Debug)]
struct PhysicalPoint(i32, i32, (u8, u8, u8)); // (x,y,color) as ints

const FRACTAL_ITERATIONS: i32 = 255;
const FRACTAL_ESCAPE: f64 = 2.0;
const WORKER_THREADS: u32 = 10;
const WINDOW_HEIGHT: i32 = 800;
const WINDOW_WIDTH: i32 = 800;
const INIT_VIRTUAL_GRID_XMIN: f64 = -2.0;
const INIT_VIRTUAL_GRID_XMAX: f64 = 2.0;
const INIT_VIRTUAL_GRID_YMIN: f64 = -2.0;
const INIT_VIRTUAL_GRID_YMAX: f64 = 2.0;

struct Mandelbrot {
    xmin: f64,
    xmax: f64,
    ymin: f64,
    ymax: f64,
    width: i32,
    height: i32,
}

impl Mandelbrot {
    pub fn new() -> Mandelbrot {
        Mandelbrot { 
            xmin: INIT_VIRTUAL_GRID_XMIN,
            xmax: INIT_VIRTUAL_GRID_XMAX,
            ymin: INIT_VIRTUAL_GRID_YMIN,
            ymax: INIT_VIRTUAL_GRID_YMAX,
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
        }
    }

    fn calc_mandelbrot_point(coord: &VirtualPoint) -> Option<VirtualPoint> {
        let mut prev_x: f64 = coord.0;
        let mut prev_y: f64 = coord.1;
        let mut escape_count: i32 = 0;
        for count in 0..FRACTAL_ITERATIONS {
           escape_count = count;
           let x = (prev_x * prev_x) - (prev_y * prev_y) + coord.0;
           let y =  (2.0 * (prev_x * prev_y)) + coord.1;
           let dist = (x * x + y * y).sqrt();
           prev_x = x;
           prev_y = y;
           if dist > FRACTAL_ESCAPE {
               break;
           } 
        }
        if escape_count > 0 && escape_count < FRACTAL_ITERATIONS-1 {
            let color = (cmp::min(255, escape_count * 10) as u8, escape_count as u8, escape_count as u8);
            return Some(VirtualPoint(coord.0, coord.1, color));
        }
        return None;
    }

    fn calc_mandelbrot_worker(points: Vec<VirtualPoint>) -> Vec<VirtualPoint> {
        //println!("Worker Start Size: {}",points.len());
        let mut results = Vec::<VirtualPoint>::new();
        for point in points {
            let result = Mandelbrot::calc_mandelbrot_point(&point);
            match result {
                Some(calc_point) => results.push(calc_point),
                None => ()
            }
        }
        //println!("Worker Stop Size: {}",results.len());
        return results;
    }

    fn draw_mandelbrot_init(&self) -> Vec<VirtualPoint> {
        //println!("0a");
        let mut points = Vec::<VirtualPoint>::new();
        let delta_x: f64 = (self.xmax - self.xmin) / self.width as f64;
        let delta_y: f64 = (self.ymax - self.ymin) / self.height as f64;
        let mut x: f64 = self.xmin;
        while x <= self.xmax {
            let mut y: f64 = self.ymin;
            while y <= self.ymax {
                points.push(VirtualPoint(x, y, (0, 0, 0)));
                y += delta_y;
            }
            x += delta_x;
        }
        //println!("0b");
        //println!("  Size={}", points.len());
        return points;
    }

    fn draw_mandelbrot_run(&self, points: Vec<VirtualPoint>) -> Vec<VirtualPoint>{
        //println!("1a");
        let mut threads = Vec::<thread::JoinHandle<Vec::<VirtualPoint>>>::new();
        for block in 0..WORKER_THREADS {
            let start_range = (points.len() / WORKER_THREADS as usize) * block as usize;
            let end_range = (points.len() / WORKER_THREADS as usize) * (block + 1) as usize;
            let mut subset = Vec::<VirtualPoint>::new();
            //println!("1b-{} Size: {}", block, subset.len());
            for index in start_range..end_range {
                subset.push(points[index]);
            }
            let worker = thread::spawn(|| {
                let result = Mandelbrot::calc_mandelbrot_worker(subset);
                result
            });
            threads.push(worker);
        }
        //println!("1b");
        let mut results = Vec::<VirtualPoint>::new();
        for thread_handle in threads {
            let result = thread_handle.join().unwrap();
            results.extend(result);
        }
        //println!("1c");
        //println!("   Size={}",results.len());
        return results;
    }

    fn draw_mandelbrot(&self) -> Vec<PhysicalPoint> {
        let points = self.draw_mandelbrot_init();
        let calc_points = self.draw_mandelbrot_run(points);
        //println!("2a");
        let mut drawing = Vec::<PhysicalPoint>::new();
        for point in calc_points {
            let x = ((point.0 - self.xmin) / (self.xmax - self.xmin) * self.width as f64) as i32;
            let y = ((point.1 - self.ymin) / (self.ymax - self.ymin) * self.height as f64) as i32;
            let color = point.2;
            let drawing_point = PhysicalPoint(x, y, color);
            drawing.push(drawing_point);
        }
        //println!("2b");
        return drawing;
    }

    fn zoom(&mut self, x:i32, y:i32, ratio:f64) {
        let virtual_grid_x_size = ((self.xmax - self.xmin) / 2.0) * ratio;
        let virtual_grid_y_size = ((self.ymax - self.ymin) / 2.0) * ratio;
        let virtual_x = (x as f64 / self.width as f64) * (self.xmax - self.xmin) + self.xmin;
        let virtual_y = (y as f64 / self.height as f64) * (self.ymax - self.ymin) + self.ymin;
        self.xmin = virtual_x - virtual_grid_x_size;
        self.xmax = virtual_x + virtual_grid_x_size;
        self.ymin = virtual_y - virtual_grid_y_size;
        self.ymax = virtual_y + virtual_grid_y_size;
    }

}


fn main() {
    let mut mandelbrot = Mandelbrot::new();
    let points = mandelbrot.draw_mandelbrot();    
    let mut app = Window::new("Mandelbrot - Rust", 800, 800);

    //println!("draw now");
    for point in points {
        let color = point.2;
        app.set_color(color.0, color.1, color.2, 255);
        app.draw_point(Point::new(point.0, point.1));
        //println!("{},{} c={:?}",point.0, point.1, point.2);
    }
    //println!("draw done");
    //app.set_color(255, 0, 255, 255);
    //app.draw_rect(Rect::new(100, 110, 120, 130));

    while app.next_frame() {
        // event handling
        while app.has_event() {
            match app.next_event() {
                // If the user clicks, we add a new Square at the position of the mouse event.
                Event::Mouse {
                    is_down: true,
                    mouse_x,
                    mouse_y,
                    ..
                } => mandelbrot.zoom(mouse_x, mouse_y, 0.8),

                _ => (),
            }
            app.clear();
            let points = mandelbrot.draw_mandelbrot();
            //println!("draw now");
            for point in points {
                let color = point.2;
                app.set_color(color.0, color.1, color.2, 255);
                app.draw_point(Point::new(point.0, point.1));
                //println!("{},{} c={:?}",point.0, point.1, point.2);
            }
            //println!("draw done");
        }

       
    }
}
