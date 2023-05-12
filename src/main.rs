use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ProcessState {
    NonExistant,
    Ready,
    Running,
    Suspended,
    Terminated,
}

#[derive(Debug, Ord, Eq, PartialEq, PartialOrd, Clone, Copy)]
struct Process {
    start_time: u64,
    execution_time: u64,
    serviced_time: u64,
    waiting_time: u64,
    priority: u64,
    id: usize,
    state: ProcessState,
}

impl Process {
    fn new(id: usize, start_time: u64, execution_time: u64, priority: u64) -> Self {
        Self {
            id,
            start_time,
            execution_time,
            priority,
            waiting_time: 0,
            serviced_time: 0,
            state: ProcessState::NonExistant,
        }
    }
}

type Strategy = fn(&mut Processor);

struct Processor {
    previous: Option<usize>,
    current: Option<usize>,
    processes: Vec<Process>,
    strategy: Strategy,
    system_time: u64,
}

impl Processor {
    fn new(processes: Vec<Process>, strategy: Strategy) -> Self {
        Self {
            previous: None,
            current: None,
            processes,
            strategy,
            system_time: 0,
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
        todo!("shortest_remaining_time")
    }

    fn round_robin(processes: Vec<Process>) -> Self {
        println!("--- Round Robin ---");
        todo!("round_robin")
    }

    fn multi_level_feedback_queue(processes: Vec<Process>) -> Self {
        println!("--- Multi Level Feedback Queue ---");
        todo!("multi_level_feedback_queue")
    }

    fn tick(&mut self) -> u64 {
        self.previous = self.current;

        for process in self
            .processes
            .iter_mut()
            .filter(|p| p.start_time == self.system_time)
        {
            process.state = ProcessState::Ready;
        }

        (&self.strategy)(self);
        self.system_time += 1;

        self.system_time
    }

    fn run(&mut self) {
        let mut log: Vec<SchedulingEvent> = Vec::new();

        while !self.executable_processes().is_empty() {
            let system_time = self.tick();

            let event = SchedulingEvent::new(
                system_time,
                self.previous_process_ref().copied(),
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
            .filter(|p| p.state == ProcessState::Ready || p.state == ProcessState::Running)
            .collect()
    }

    fn executable_processes(&self) -> Vec<&Process> {
        self.processes
            .iter()
            .filter(|p| p.state != ProcessState::Terminated)
            .collect()
    }

    fn get_process(&self, id: usize) -> Option<&Process> {
        self.processes.iter().find(|p| p.id == id)
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

    fn previous_process_ref(&self) -> Option<&Process> {
        if let Some(id) = self.previous {
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
    system_time: u64,
    start_of_tick: Option<Process>,
    end_of_tick: Option<Process>,
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
    if let Some(current) = processor.current_process_mut() {
        current.execution_time -= 1;
        current.serviced_time += 1;
        if current.execution_time > 0 {
            return;
        }

        processor.set_current_process_state(ProcessState::Terminated);
        processor.current = None;
    }

    // At this point, the processor has no currently executed process
    processor.current = match processor.scheduable_processes().first() {
        Some(process) => Some(process.id),
        None => None,
    };
    processor.set_current_process_state(ProcessState::Running);
}

fn shortest_job_first(processor: &mut Processor) {
    if let Some(current) = processor.current_process_mut() {
        current.execution_time -= 1;
        current.serviced_time += 1;
        if current.execution_time > 0 {
            return;
        }

        processor.set_current_process_state(ProcessState::Terminated);
        processor.current = None;
    }

    // At this point, the processor has no currently executed process
    let (mut id, mut len) = (None, None);
    for process in processor.scheduable_processes() {
        if process.execution_time < len.unwrap_or(u64::MAX) {
            (id, len) = (Some(process.id), Some(process.execution_time));
        }
    }

    processor.current = id;
    processor.set_current_process_state(ProcessState::Running);
}

fn highest_priority_first(processor: &mut Processor) {
    if let Some(current) = processor.current_process_mut() {
        current.execution_time -= 1;
        current.serviced_time += 1;
        if current.execution_time > 0 {
            return;
        }

        processor.set_current_process_state(ProcessState::Terminated);
        processor.current = None;
    }

    let mut scheduable = processor.scheduable_processes();
    scheduable.sort_by_key(|p| p.priority);

    let id = match scheduable.first() {
        Some(process) => Some(process.id),
        None => None
    };

    processor.current = id;
    processor.set_current_process_state(ProcessState::Running);
}

fn highest_priority_first_preemptive(processor: &mut Processor) {
    todo!();
}

fn main() {
    let mut processes = vec![
        Process::new(0, 1, 2, 2),
        Process::new(1, 3, 3, 3),
        Process::new(2, 3, 1, 5),
        Process::new(3, 0, 4, 1),
        Process::new(4, 2, 2, 1),
    ];

    processes.sort();

    Processor::first_come_first_serve(processes.clone()).run();
    Processor::shortest_job_first(processes.clone()).run();
    Processor::highest_priority_first(processes.clone()).run();
}
