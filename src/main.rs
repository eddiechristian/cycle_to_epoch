extern crate cpu_cycle_counter;
extern crate libc;

use std::mem;

pub const NANO_SECONDS_IN_SEC: i64 = 1000000000;

fn main() {
    let tsc = unsafe { cpu_cycle_counter::rdtscp() };
    let ts = get_ts();
    println!("tsc: {:?}\nts\n\ttv_sec: {:?} \n\ttv_nsec: {:?}",tsc,ts.tv_sec, ts.tv_nsec);
    let mut clk = CgtClock::default();
    let mut timespec: libc::timespec = unsafe { mem::uninitialized()};
    clk.get_cycle_time_as_ts(&mut timespec);
    println!("timespec: tv_sec: {:?} tv_nsec: {:?}",timespec.tv_sec, timespec.tv_nsec);
    clk.get_cycle_time_as_ts(&mut timespec);
    println!("timespec: tv_sec: {:?} tv_nsec: {:?}",timespec.tv_sec, timespec.tv_nsec);
}

fn get_ts_diff(ts1 : &mut libc::timespec, ts2 : &mut libc::timespec) ->  libc::timespec {
    let mut ts: libc::timespec = unsafe { mem::uninitialized()};
    ts.tv_sec = ts1.tv_sec - ts2.tv_sec;
    ts.tv_nsec = ts1.tv_nsec - ts2.tv_nsec;
    if ts.tv_nsec < 0 {
         ts.tv_sec = ts.tv_sec - 1;
         ts.tv_nsec = ts.tv_nsec + NANO_SECONDS_IN_SEC;
    }
    ts
}

#[cfg(target_os = "linux")]
pub fn get_ts() -> libc::timespec {
    unsafe {
        let mut timespec: libc::timespec = mem::uninitialized();
        let ret = libc::clock_gettime(libc::CLOCK_REALTIME, &mut timespec);
        if ret != 0 { panic!("clock_gettime failed"); }
        timespec
    }
}

#[cfg(target_os = "linux")]
pub struct CgtClock {
    ticks_per_nanosec: f64,
    calibrated: bool,
}

impl Default for CgtClock {
    fn default() -> CgtClock {
        CgtClock {
            ticks_per_nanosec: 0.0,
            calibrated: false,
        }
    }
}

#[cfg(target_os = "linux")]
impl CgtClock {
    fn calibrate_ticks(&mut self) {
        println!("calibrating", );
        let mut begin_ts = get_ts();
        let begin_tsc = unsafe { cpu_cycle_counter::rdtscp() };
        let mut total: i64 = 0;
        for x in 0..1000000 {
            total = total + 1;
        }
        let mut end_ts = get_ts();
        let end_tsc = unsafe { cpu_cycle_counter::rdtscp() };
        let tmp_ts = get_ts_diff(&mut end_ts,&mut begin_ts);
        let nsec_elapsed : i64 = tmp_ts.tv_sec * 1000000000 + tmp_ts.tv_nsec;
        self.ticks_per_nanosec = (end_tsc - begin_tsc) as f64 /nsec_elapsed as f64;
        println!("ticks_per_nanosec {:?}", self.ticks_per_nanosec);
    }


    pub fn get_cycle_time_as_ts(&mut self,
                                ts : &mut libc::timespec) {
        if !self.calibrated{
            self.calibrate_ticks();
            self.calibrated = true;
        }
        let tsc = unsafe { cpu_cycle_counter::rdtscp() };
        let tsc_64 =  tsc as f64;
        println!("ed tsc {:?} tsc_64 {:?}", tsc,tsc_64);
        
        let denominator = tsc_64 /self.ticks_per_nanosec;
        self.get_time_spec(ts,denominator as i64);
    }

    fn get_time_spec(&mut self,
                     ts : &mut libc::timespec,
                     nsecs: i64) {
        ts.tv_sec = nsecs / NANO_SECONDS_IN_SEC;
        ts.tv_nsec = nsecs % NANO_SECONDS_IN_SEC;
    }

}
