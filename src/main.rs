use std::{fmt, os::unix::process};

#[derive(Debug, Clone, Copy, PartialEq)]
enum ProcessState {
    NonExistant,
    Ready,
    Running,
    Suspended,
    Terminated,
}

#[derive(Debug, Clone, Copy)]
struct Process {
    start_time: u64, // System time, at which this process can be set to the READY state
    remaining_time: u64, // Ticks the process needs until it is completed
    serviced_time: u64, // Ticks the process has already been serviced
    waiting_time: u64, // How many ticks the process is waiting, either in the READY or SUSPENDED state
    priority: u64,     // Priority of the process. The lower the number, the higher the priority.
    id: usize,         // The unique identifier of the process
    state: ProcessState, // The state whicht the process currently is in
}

impl PartialEq for Process {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Process {}

impl Process {
    fn new(id: usize, start_time: u64, execution_time: u64, priority: u64) -> Self {
        Self {
            id,
            start_time,
            remaining_time: execution_time,
            priority,
            waiting_time: 0,
            serviced_time: 0,
            state: ProcessState::NonExistant,
        }
    }
}

type Strategy = fn(&mut Processor);

struct Processor {
    previous: Option<usize>, // The id of the process executed in the last step
    current: Option<usize>,  // The id of the process currently being executed
    processes: Vec<Process>, // All processes
    strategy: Strategy,      // The scheduling strategy to use
    system_time: u64,        // The current system time
    time_quantum: u64,       // Minimal amount of time a process is allowed to run uninterrupted
}

impl Processor {
    fn new(processes: Vec<Process>, strategy: Strategy) -> Self {
        Self {
            previous: None,
            current: None,
            processes,
            strategy,
            system_time: 0,
            time_quantum: 3,
        }
    }

    fn first_come_first_serve(processes: Vec<Process>) -> Self {
        println!("--- First Come First Serve ---");
        Self::new(processes, first_come_first_serve)
    }

    fn shortest_job_first(processes: Vec<Process>) -> Self {
        println!("--- Shortest Job First ---");
        Self::new(processes, shortest_job_first)
    }

    fn highest_priority_first(processes: Vec<Process>) -> Self {
        println!("--- Highest Priority First ---");
        Self::new(processes, highest_priority_first)
    }

    fn highest_priority_first_preemptive(processes: Vec<Process>) -> Self {
        println!("--- Highest Priority First Preemptive ---");
        Self::new(processes, highest_priority_first_preemptive)
    }

    fn shortest_remaining_time(processes: Vec<Process>) -> Self {
        println!("--- Shortest Remaining Time ---");
        Self::new(processes, shortest_remaining_time)
    }

    fn round_robin(processes: Vec<Process>) -> Self {
        println!("--- Round Robin ---");
        todo!("round_robin")
    }

    fn multi_level_feedback_queue(processes: Vec<Process>) -> Self {
        println!("--- Multi Level Feedback Queue ---");
        todo!("multi_level_feedback_queue")
    }

    fn tick(&mut self) -> (u64, Option<Process>) {

        let previous = self.current;
        
        // Set the state of all processes, that could be started in this tick to READY
        for process in self
            .processes
            .iter_mut()
            .filter(|p| p.start_time == self.system_time)
        {
            process.state = ProcessState::Ready;
        }

        (&self.strategy)(self);
        self.system_time += 1;

        let previous = match previous {
            Some(id) => self.get_process(id).copied(),
            None => None,
        };

        if let Some(process) = self.current_process_mut() {
            // As it is the current process, it did not wait in the last tick
            process.waiting_time = 0;
            // Indicate, that the current process has been serviced
            process.serviced_time += 1;
            process.remaining_time -= 1;
            // If the process has no remaining time, it terminates
            if process.remaining_time == 0 {
                process.state = ProcessState::Terminated;
            }
        }

        for process in self.executable_processes_mut() {
            process.waiting_time += 1;
        }

        (self.system_time, previous)
    }

    fn run(&mut self) {
        let mut log: Vec<SchedulingEvent> = Vec::new();

        while !self.executable_processes().is_empty() {
            let (system_time, previous_process_state) = self.tick();

            let event = SchedulingEvent::new(
                system_time - 1,
                previous_process_state,
                self.current_process_ref().copied(),
            );
            log.push(event);
        }

        for event in log {
            println!("{}", event);
        }
    }

    fn scheduable_processes(&self) -> Vec<&Process> {
        self.processes
            .iter()
            .filter(|p| {
                p.state == ProcessState::Ready
                    || p.state == ProcessState::Running
                    || p.state == ProcessState::Suspended
            })
            .collect()
    }

    fn executable_processes(&self) -> Vec<&Process> {
        self.processes
            .iter()
            .filter(|p| p.state != ProcessState::Terminated)
            .collect()
    }

    fn executable_processes_mut(&mut self) -> Vec<&mut Process> {
        self.processes
            .iter_mut()
            .filter(|p| p.state != ProcessState::Terminated)
            .collect()
    }

    fn needs_schedule(&self) -> bool {
        let terminated = if let Some(process) = self.current_process_ref() {
            process.state == ProcessState::Terminated
        } else {
            false
        };
        self.current_process_ref().is_none() || terminated
    }

    fn get_process(&self, id: usize) -> Option<&Process> {
        self.processes.iter().find(|p| p.id == id)
    }

    fn suspend_current(&mut self) {
        if let Some(cur) = self.current_process_mut() {
            if cur.state != ProcessState::Terminated && cur.state != ProcessState::Ready {
                cur.state = ProcessState::Suspended;
            }
        }
    }

    fn set_current(&mut self, id: Option<usize>) {
        self.current = id;
        self.set_current_process_state(ProcessState::Running);
    }

    fn current_process_mut(&mut self) -> Option<&mut Process> {
        if let Some(id) = self.current {
            return Some(self.processes.iter_mut().find(|p| p.id == id).unwrap());
        }
        None
    }

    fn current_process_ref(&self) -> Option<&Process> {
        if let Some(id) = self.current {
            return self.get_process(id);
        }
        None
    }

    fn set_current_process_state(&mut self, state: ProcessState) {
        if let Some(process_mut) = self.current_process_mut() {
            process_mut.state = state;
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct SchedulingEvent {
    system_time: u64,               // Tick, for which this event is generated
    start_of_tick: Option<Process>, // The process being executed at the start of the tick
    end_of_tick: Option<Process>,   // The process being executed at the end of the tick
}

impl SchedulingEvent {
    fn new(system_time: u64, start_of_tick: Option<Process>, end_of_tick: Option<Process>) -> Self {
        Self {
            system_time,
            start_of_tick,
            end_of_tick,
        }
    }
}

impl fmt::Display for SchedulingEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:05}: ", self.system_time)?;
        match self.start_of_tick {
            Some(process) => write!(f, "{:?}({})", process.state, process.id)?,
            None => write!(f, "None")?,
        };
        write!(f, " -> ")?;
        match self.end_of_tick {
            Some(process) => write!(f, "{:?}({})", process.state, process.id)?,
            None => write!(f, "None")?,
        };
        Ok(())
    }
}

fn first_come_first_serve(processor: &mut Processor) {
    // We only have to schedule a new process, if there is non currently running or the last process has terminated
    if processor.needs_schedule() {
        // Pick the unterminated process with the earliest start time
        let mut scheduable = processor.scheduable_processes();
        scheduable.sort_by_key(|&p| p.start_time);
        let id = match scheduable.first() {
            Some(process) => Some(process.id),
            None => None,
        };

        processor.set_current(id);
    }
}

fn shortest_job_first(processor: &mut Processor) {
    // We only have to schedule a new process, if there is non currently running or the last process has terminated
    if processor.needs_schedule() {
        // Pick the unterminated process with the lowest execution time
        let mut scheduable = processor.scheduable_processes();
        scheduable.sort_by_key(|&p| p.remaining_time);
       
        let id  = match scheduable.first() {
            Some(process) => Some(process.id),
            None => None,
        };

        processor.set_current(id);
    }
}

fn highest_priority_first(processor: &mut Processor) {
    // We only have to schedule a new process, if there is non currently running or the last process has terminated
    if processor.needs_schedule() {
        // If there is no running process, start the process with the highest priority
        let mut scheduable = processor.scheduable_processes();
        scheduable.sort_by_key(|p| p.priority);

        let id = match scheduable.first() {
            Some(process) => Some(process.id),
            None => None,
        };

        processor.set_current(id);
    }
}

fn highest_priority_first_preemptive(processor: &mut Processor) {
    processor.suspend_current();

    // Get the process with the highest priority
    let mut scheduable = processor.scheduable_processes();
    scheduable.sort_by_key(|&p| p.priority);

    let id = match scheduable.first() {
        Some(process) => Some(process.id),
        None => None,
    };   

    processor.set_current(id);
}

fn shortest_remaining_time(processor: &mut Processor) {
    processor.suspend_current();

    // Get the process with the shortest remaining time
    let mut scheduable = processor.scheduable_processes();
    scheduable.sort_by_key(|&p| p.remaining_time);

    let id = match scheduable.first() {
        Some(process) => Some(process.id),
        None => None,
    };   

    processor.set_current(id);
}

fn round_robin(processor: &mut Processor) {
    todo!()
}

fn main() {
    /*      0    5    10   15   20   25   30   35   40   45   50
            |    |    |    |    |    |    |    |    |    |    |
        0:  ###############....................................
        1:  ...######################..........................
        2:  ...####............................................
        3:  ........#################..........................
        4:  ..................######################...........
        5:  ........................................###########
    */
    let processes = vec![
        Process::new(0, 0, 15, 1),
        Process::new(1, 3, 22, 2),
        Process::new(2, 3, 4, 3),
        Process::new(3, 8, 17, 0),
        Process::new(4, 18, 22, 5),
        Process::new(5, 40, 10, 4),
    ];

    Processor::first_come_first_serve(processes.clone()).run();
    Processor::shortest_job_first(processes.clone()).run();
    Processor::highest_priority_first(processes.clone()).run();
    Processor::highest_priority_first_preemptive(processes.clone()).run();
    Processor::shortest_remaining_time(processes.clone()).run()
}
