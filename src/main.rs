use std::borrow::BorrowMut;

#[derive(Debug, Ord, Eq, PartialEq, PartialOrd, Clone, Copy)]
struct Process {
    start_time: u64,
    remaining_time: u64,
    serviced_time: u64,
    waiting_time: u64,
    priority: u64,
    id: usize,
}

impl Process {
    fn new(id: usize, start_time: u64, execution_time: u64, priority: u64) -> Self {
        Self {
            id,
            start_time,
            remaining_time: execution_time,
            priority,
            waiting_time: 0,
            serviced_time: 0,
        }
    }
}

type Strategy = fn(&mut Processor);

struct Processor {
    current: Option<usize>,
    processes: Vec<Process>,
    strategy: Strategy,
    time: u64,
}

impl Processor {
    fn new(processes: Vec<Process>, strategy: Strategy) -> Self {
        Self {
            current: None,
            processes,
            strategy,
            time: 0,
        }
    }

    fn tick(&mut self) {
        (&self.strategy)(self);
        self.time += 1;
    }
}

#[derive(Debug)]
enum Event {
    Started,
    Running,
    Finished,
    Interrupted,
    Resumed,
    ProcessorIdle,
}

#[derive(Debug)]
struct SchedulingEvent {
    time: u64,
    process: Option<usize>,
    event: Event,
}

impl SchedulingEvent {
    fn from(previous_process: Option<usize>, processor: &Processor) -> Vec<SchedulingEvent> {
        let mut local_log = vec![];
        match previous_process {
            None => {
                if let Some(_) = processor.current {
                    local_log.push(SchedulingEvent {
                        time: processor.time,
                        process: processor.current,
                        event: Event::Started,
                    });
                } else {
                    local_log.push(SchedulingEvent {
                        time: processor.time,
                        process: processor.current,
                        event: Event::ProcessorIdle,
                    });
                }
            }
            Some(prev_id) => match processor.current {
                None => {
                    local_log.push(SchedulingEvent {
                        time: processor.time,
                        process: Some(prev_id),
                        event: Event::Finished,
                    });
                }
                Some(cur_id) => {
                    if prev_id == cur_id {
                        local_log.push(SchedulingEvent {
                            time: processor.time,
                            process: processor.current,
                            event: Event::Running,
                        });
                    } else {
                        let cur_proc = processor.processes.iter().find(|p| p.id == cur_id).unwrap();
                        let prev_proc = processor
                            .processes
                            .iter()
                            .find(|p| p.id == prev_id)
                            .unwrap();
                        if prev_proc.remaining_time != 0 {
                            local_log.push(SchedulingEvent {
                                time: processor.time,
                                process: Some(prev_id),
                                event: Event::Interrupted,
                            });
                            if cur_proc.serviced_time != 0 {
                                local_log.push(SchedulingEvent {
                                    time: processor.time,
                                    process: Some(cur_id),
                                    event: Event::Resumed,
                                });
                            } else {
                                local_log.push(SchedulingEvent {
                                    time: processor.time,
                                    process: Some(cur_id),
                                    event: Event::Started,
                                });
                            }
                        } else {
                            local_log.push(SchedulingEvent {
                                time: processor.time,
                                process: Some(prev_id),
                                event: Event::Finished,
                            });
                            local_log.push(SchedulingEvent {
                                time: processor.time,
                                process: Some(cur_id),
                                event: Event::Started,
                            });
                        }
                    }
                }
            },
        }
        local_log
    }
}

fn first_come_first_serve(processor: &mut Processor) {
    if let Some(id) = processor.current {
        let mut current = processor.processes.iter_mut().find(|p| p.id == id).unwrap();

        current.remaining_time -= 1;
        current.serviced_time += 1;
        if current.remaining_time > 0 {
            return;
        }

        processor.current = None;
    }
    // At this point, the processor has no currently executed process
    for process in &processor.processes {
        if process.remaining_time > 0 {
            processor.current = Some(process.id);
        }
    }
}

fn shortes_job_first(processor: &mut Processor) {
    if let Some(id) = processor.current {
        let mut current = processor.processes.iter_mut().find(|p| p.id == id).unwrap();

        current.remaining_time -= 1;
        current.serviced_time += 1;
        if current.remaining_time > 0 {
            return;
        }

        processor.current = None;
    }

    let (mut id, mut len) = (None, u64::MAX);
    for process in processor.processes.iter().filter(|p| p.remaining_time != 0) {
        if process.remaining_time < len {
            (id, len) = (Some(process.id), process.remaining_time);
        }
    }
    processor.current = id;
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

    let mut log: Vec<SchedulingEvent> = Vec::new();
    let mut processor = Processor::new(processes, first_come_first_serve);

    for _ in 0..30 {
        let previous = processor.current;
        processor.tick();
        log.append(&mut SchedulingEvent::from(previous, &processor));
    }

    println!("{:#?}", log);
}
