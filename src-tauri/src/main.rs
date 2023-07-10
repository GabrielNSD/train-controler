// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    sync::{Arc, Mutex},
    thread,
};

use once_cell::sync::Lazy;

static mut TRAIN_SPEEDS: Lazy<Vec<i32>> = Lazy::new(|| vec![0, 0, 0, 0]);

static mut TRAIN_POSITIONS: Lazy<Vec<(f32, f32)>> =
    Lazy::new(|| vec![(0.0, 0.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0)]);

#[derive(Debug)]
struct Track {
    id: u32,
    start: (f32, f32),
    end: (f32, f32),
    mutex: Option<Arc<Mutex<i32>>>,
}

impl Track {
    pub fn new(id: u32, start: (f32, f32), end: (f32, f32)) -> Self {
        Self {
            id,
            start,
            end,
            mutex: None,
        }
    }

    pub fn set_mutex(&mut self, mutex: Arc<Mutex<i32>>) {
        self.mutex = Some(mutex);
    }
}

#[derive(Debug)]
struct Loop {
    tracks: Vec<Track>,
}

impl Loop {
    pub fn new(tracks: Vec<Track>) -> Self {
        Self { tracks }
    }

    pub fn get_next_track(&self, current_track_id: u32) -> &Track {
        let mut next_track = current_track_id + 1;
        if next_track >= self.tracks.len() as u32 {
            next_track = 0;
        }
        &self.tracks[next_track as usize]
    }

    pub fn get_second_next_track(&self, current_track_id: u32) -> &Track {
        let mut second_next_track = current_track_id + 2;
        if second_next_track >= self.tracks.len() as u32 {
            second_next_track = 0;
        }
        &self.tracks[second_next_track as usize]
    }
}

#[derive(Debug)]
struct Train {
    id: u32,
    position: (f32, f32),
    track_loop: Loop,
    current_track: u32,
    greedy: bool,
}

impl Train {
    pub fn new(id: u32, track_loop: Loop, greedy: bool) -> Self {
        let first_track = &track_loop.tracks[0];
        let position = first_track.start;
        let current_track: u32 = first_track.id;

        Self {
            id,
            position,
            track_loop,
            current_track,
            greedy,
        }
    }

    fn is_vertical(&mut self, track_id: usize) -> bool {
        if self.track_loop.tracks[track_id].start.0 == self.track_loop.tracks[track_id].end.0 {
            true
        } else {
            false
        }
    }

    fn magnitude_multiplier(&mut self, track_id: usize) -> i32 {
        let is_vertical = self.is_vertical(track_id);

        if is_vertical {
            if self.track_loop.tracks[track_id].start.1 < self.track_loop.tracks[track_id].end.1 {
                1
            } else {
                -1
            }
        } else {
            if self.track_loop.tracks[track_id].start.0 < self.track_loop.tracks[track_id].end.0 {
                1
            } else {
                -1
            }
        }
    }

    pub fn move_train(&mut self, speed: i32) {
        // check in which track the train is
        let current_track = self.current_track;

        let invert_direction = self.magnitude_multiplier(current_track as usize);

        // time elapsed is constant (1sec)

        // calculate distace traveled
        let distance = speed as f32;

        // calculate new position
        let mut new_position = self.position;
        let is_vertical = self.is_vertical(current_track as usize);
        if !is_vertical {
            // move in x direction
            new_position.0 += distance * invert_direction as f32;
        } else {
            // move in y direction
            new_position.1 += distance * invert_direction as f32;
        }

        // check if train has reached end of track
        let current_track = &self.track_loop.tracks[current_track as usize];
        let mut reached_end = false;
        let mut remmainer = 0.0;

        if new_position == current_track.end {
            reached_end = true;
        } else {
            if invert_direction < 0
                && (new_position.0 < current_track.end.0 || new_position.1 < current_track.end.1)
            {
                reached_end = true;
                if is_vertical {
                    remmainer = (new_position.1 - current_track.end.1) * -1.0;
                } else {
                    remmainer = (new_position.0 - current_track.end.0) * -1.0;
                }
            } else if invert_direction > 0
                && (new_position.0 > current_track.end.0 || new_position.1 > current_track.end.1)
            {
                reached_end = true;
                if is_vertical {
                    remmainer = new_position.1 - current_track.end.1;
                } else {
                    remmainer = new_position.0 - current_track.end.0;
                }
            }
        }

        if reached_end {
            self.position = current_track.end;
            new_position = self.position;

            // unlock own mutex if exists
            if current_track.mutex.is_some() {
                let current_track_mutex = current_track.mutex.as_ref().unwrap();
                let mut locked = current_track_mutex.lock().unwrap();

                if *locked == self.id as i32 {
                    *locked = 100;
                }
            }

            // check if the next track has a mutex
            let next_track = self.track_loop.get_next_track(current_track.id);
            if next_track.mutex.is_some() {
                // check if the mutex is locked
                let mutex = next_track.mutex.as_ref().unwrap();
                let mut locked = mutex.lock().unwrap();

                let mut next_track_locked = false;
                let mut second_next_track_locked = false;

                if *locked != 100 && *locked != self.id as i32 {
                    next_track_locked = true;
                }

                if self.greedy {
                    let second_next_track = self.track_loop.get_second_next_track(current_track.id);
                    if second_next_track.mutex.is_some() {
                        let second_next_track_mutex = second_next_track.mutex.as_ref().unwrap();
                        let second_next_locked = second_next_track_mutex.lock().unwrap();

                        if *second_next_locked != 100 && *second_next_locked != self.id as i32 {
                            second_next_track_locked = true;
                        }
                    }
                }

                // if the next track is unlocked, lock it
                if !next_track_locked && !second_next_track_locked {
                    // lock mutex
                    *locked = self.id as i32;

                    //lock the next track if it is a greedy train
                    if self.greedy {
                        let second_next_track =
                            self.track_loop.get_second_next_track(current_track.id);
                        if second_next_track.mutex.is_some() {
                            let second_next_track_mutex = second_next_track.mutex.as_ref().unwrap();
                            let mut second_next_locked = second_next_track_mutex.lock().unwrap();

                            *second_next_locked = self.id as i32;
                        }
                    }

                    self.position = next_track.start;

                    self.current_track = next_track.id;
                }
            } else {
                // set current track to next track
                self.current_track = next_track.id;
                let is_vertical = self.is_vertical(self.current_track as usize);
                let invert_direction = self.magnitude_multiplier(self.current_track as usize);
                if !is_vertical {
                    // move in x direction
                    new_position.0 += remmainer * invert_direction as f32;
                } else {
                    // move in y direction
                    new_position.1 += remmainer * invert_direction as f32;
                }

                self.position = new_position;
            }
        }

        self.position = new_position;

        unsafe {
            TRAIN_POSITIONS[self.id as usize] = self.position;
        }
    }
}

#[tauri::command]
fn get_trains() -> Vec<(f32, f32)> {
    unsafe { TRAIN_POSITIONS.clone() }
}

#[tauri::command]
fn set_train_speed(train_id: usize, speed: i32) {
    unsafe {
        TRAIN_SPEEDS[train_id] = speed;
    }
}

fn main() {
    // mutex indexed in clockwise direction
    let mutex_1 = Arc::new(Mutex::new(100));
    let mutex_2 = Arc::new(Mutex::new(100));
    let mutex_3 = Arc::new(Mutex::new(100));
    let mutex_4 = Arc::new(Mutex::new(100));

    // mutex for loop 1
    let clone_mutex_1_1 = Arc::clone(&mutex_1);
    let clone_mutex_4_1 = Arc::clone(&mutex_4);

    //mutex for loop 2
    let clone_mutex_1_2 = Arc::clone(&mutex_1);
    let clone_mutex_2_2 = Arc::clone(&mutex_2);

    //mutex for loop 3
    let clone_mutex_3_3 = Arc::clone(&mutex_3);
    let clone_mutex_4_3 = Arc::clone(&mutex_4);

    //mutex for loop 4
    let clone_mutex_2_4 = Arc::clone(&mutex_2);
    let clone_mutex_3_4 = Arc::clone(&mutex_3);

    // first loop
    let track_1 = Track::new(0, (100.0, 300.0), (100.0, 100.0));
    let track_2 = Track::new(1, (100.0, 100.0), (500.0, 100.0));
    let mut track_3 = Track::new(2, (500.0, 100.0), (500.0, 300.0));
    let mut track_4 = Track::new(3, (500.0, 300.0), (100.0, 300.0));

    thread::spawn(move || {
        track_3.set_mutex(clone_mutex_1_1);
        track_4.set_mutex(clone_mutex_4_1);

        let loop_1 = Loop::new(vec![track_1, track_2, track_3, track_4]);

        let mut train_0 = Train::new(0, loop_1, true);
        loop {
            let speeds = unsafe { TRAIN_SPEEDS.clone() };

            // sleep for 1 sec
            thread::sleep(std::time::Duration::from_secs(1));
            train_0.move_train(speeds[0]);
        }
    });

    // second loop
    let track_1_2 = Track::new(0, (900.0, 100.0), (900.0, 300.0));
    let mut track_2_2 = Track::new(1, (900.0, 300.0), (500.0, 300.0));
    let mut track_3_2 = Track::new(2, (500.0, 300.0), (500.0, 100.0));
    let track_4_2 = Track::new(3, (500.0, 100.0), (900.0, 100.0));

    thread::spawn(move || {
        track_2_2.set_mutex(clone_mutex_2_2);
        track_3_2.set_mutex(clone_mutex_1_2);

        let loop_2 = Loop::new(vec![track_1_2, track_2_2, track_3_2, track_4_2]);
        let mut train_1 = Train::new(1, loop_2, false);
        loop {
            let speeds = unsafe { TRAIN_SPEEDS.clone() };
            // sleep for 1 sec
            thread::sleep(std::time::Duration::from_secs(1));

            train_1.move_train(speeds[1]);
        }
    });

    //third loop
    let track_1_3 = Track::new(0, (100.0, 500.0), (100.0, 300.0));
    let mut track_2_3 = Track::new(1, (100.0, 300.0), (500.0, 300.0));
    let mut track_3_3 = Track::new(2, (500.0, 300.0), (500.0, 500.0));
    let track_4_3 = Track::new(3, (500.0, 500.0), (100.0, 500.0));

    thread::spawn(move || {
        track_2_3.set_mutex(clone_mutex_4_3);
        track_3_3.set_mutex(clone_mutex_3_3);

        let loop_3 = Loop::new(vec![track_1_3, track_2_3, track_3_3, track_4_3]);
        let mut train_2 = Train::new(2, loop_3, false);
        loop {
            let speeds = unsafe { TRAIN_SPEEDS.clone() };
            // sleep for 1 sec
            thread::sleep(std::time::Duration::from_secs(1));

            train_2.move_train(speeds[2]);
        }
    });

    // fourth loop
    let track_1_4 = Track::new(0, (900.0, 500.0), (500.0, 500.0));
    let mut track_2_4 = Track::new(1, (500.0, 500.0), (500.0, 300.0));
    let mut track_3_4 = Track::new(2, (500.0, 300.0), (900.0, 300.0));
    let track_4_4 = Track::new(3, (900.0, 300.0), (900.0, 500.0));

    thread::spawn(move || {
        track_2_4.set_mutex(clone_mutex_3_4);
        track_3_4.set_mutex(clone_mutex_2_4);

        let loop_4 = Loop::new(vec![track_1_4, track_2_4, track_3_4, track_4_4]);
        let mut train_3 = Train::new(3, loop_4, true);
        loop {
            let speeds = unsafe { TRAIN_SPEEDS.clone() };
            // sleep for 1 sec
            thread::sleep(std::time::Duration::from_secs(1));

            train_3.move_train(speeds[3]);
        }
    });

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_trains, set_train_speed])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use crate::{Loop, Track, Train};

    #[test]
    fn test() {
        println!("{:?}", Track::new(0, (100.0, 300.0), (100.0, 100.0)));
    }

    #[test]
    fn create_loop() {
        let track_1 = Track::new(0, (100.0, 300.0), (100.0, 100.0));
        let track_2 = Track::new(1, (100.0, 100.0), (500.0, 100.0));
        let track_3 = Track::new(2, (500.0, 100.0), (500.0, 300.0));
        let track_4 = Track::new(3, (500.0, 300.0), (100.0, 300.0));

        let loop_1 = Loop::new(vec![track_1, track_2, track_3, track_4]);

        println!("{:?}", loop_1);

        assert_eq!(loop_1.tracks[0].id, 0);
        assert_eq!(loop_1.tracks[1].id, 1);
        assert_eq!(loop_1.tracks[2].id, 2);
        assert_eq!(loop_1.tracks[3].id, 3);
        assert_eq!(loop_1.tracks[0].start, (100.0, 300.0));
        assert_eq!(loop_1.tracks[0].end, (100.0, 100.0));
        assert_eq!(loop_1.tracks[1].start, (100.0, 100.0));
        assert_eq!(loop_1.tracks[1].end, (500.0, 100.0));
        assert_eq!(loop_1.tracks[2].start, (500.0, 100.0));
        assert_eq!(loop_1.tracks[2].end, (500.0, 300.0));
        assert_eq!(loop_1.tracks[3].start, (500.0, 300.0));
        assert_eq!(loop_1.tracks[3].end, (100.0, 300.0));
    }

    #[test]
    fn create_train() {
        let track_1 = Track::new(0, (100.0, 300.0), (100.0, 100.0));
        let track_2 = Track::new(1, (100.0, 100.0), (500.0, 100.0));
        let track_3 = Track::new(2, (500.0, 100.0), (500.0, 300.0));
        let track_4 = Track::new(3, (500.0, 300.0), (100.0, 300.0));

        let loop_1 = Loop::new(vec![track_1, track_2, track_3, track_4]);

        let mut train_1 = Train::new(0, loop_1, false);

        assert_eq!(train_1.id, 0);
        assert_eq!(train_1.position, (100.0, 300.0));

        for _i in 1..10 {
            train_1.move_train(10);
        }

        assert_eq!(train_1.position, (100.0, 210.0));

        for _i in 1..5 {
            train_1.move_train(50);
        }

        assert_eq!(train_1.current_track, 1);
        assert_eq!(train_1.position, (190.0, 100.0));
    }
}
