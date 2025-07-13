use core::arch::{asm, naked_asm};

const STACK_SIZE: u32 = 64 * 1024;
const MAX_TASKS: u32 = 125;

#[derive(Copy, Clone, Debug)]
pub struct Task {
    pub kernel_stack: u32,
    pub stack: u32,
    pub cpu_state_ptr: u32,
    pub state: TaskState,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TaskState {
    Null,
    Ready,
    Zombie,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct CPUState {
    eax: u32,
    ebx: u32,
    ecx: u32,
    edx: u32,
    esi: u32,
    edi: u32,
    ebp: u32,

    eip: u32,
    cs: u32,
    eflags: u32,
    esp: u32,
    ss: u32,
}

static NULL_TASK: Task = Task {
    stack: 0,
    kernel_stack: 0,
    cpu_state_ptr: 0,
    state: TaskState::Null,
};

impl Task {
    pub fn init(&mut self, entry_point: u32, args: Option<&[u32]>) {
        self.state = TaskState::Ready;
        self.stack = unsafe { (*(&raw mut crate::pmm::PADDR)).malloc(STACK_SIZE).unwrap() };

        let state = unsafe {
            (self.stack as *mut u8).add(STACK_SIZE as usize - core::mem::size_of::<CPUState>())
        } as *mut CPUState;
        self.cpu_state_ptr = state as u32;

        let mut arg_count = 0;
        if args.is_some() {
            arg_count = core::cmp::min(args.unwrap().len(), 4);
        }

        unsafe {
            (*state).eax = 0;
            (*state).ebx = if arg_count > 0 { args.unwrap()[0] } else { 0 };
            (*state).ecx = if arg_count > 1 { args.unwrap()[1] } else { 0 };
            (*state).edx = if arg_count > 2 { args.unwrap()[2] } else { 0 };
            (*state).esi = if arg_count > 3 { args.unwrap()[3] } else { 0 };

            (*state).edi = 0;
            (*state).ebp = 0;
            (*state).esp = state as u32;
            (*state).eip = entry_point;
            (*state).cs = 0x8;
            (*state).eflags = 0x202;
            (*state).ss = 0x10;
        }
    }

    pub fn init_u(&mut self, entry_point: u32, args: Option<&[u32]>) {
        self.state = TaskState::Ready;

        unsafe {
            self.stack = (*(&raw mut crate::pmm::PADDR)).malloc(STACK_SIZE).unwrap();
            self.kernel_stack =
                (*(&raw mut crate::pmm::PADDR)).malloc(STACK_SIZE).unwrap() + STACK_SIZE;
        }

        let state = unsafe {
            (self.stack as *mut u8).add(STACK_SIZE as usize - core::mem::size_of::<CPUState>())
        } as *mut CPUState;
        self.cpu_state_ptr = state as u32;

        let mut arg_count = 0;
        if args.is_some() {
            arg_count = core::cmp::min(args.unwrap().len(), 4);
        }

        unsafe {
            (*state).eax = 0;
            (*state).ebx = if arg_count > 0 { args.unwrap()[0] } else { 0 };
            (*state).ecx = if arg_count > 1 { args.unwrap()[1] } else { 0 };
            (*state).edx = if arg_count > 2 { args.unwrap()[2] } else { 0 };
            (*state).esi = if arg_count > 3 { args.unwrap()[3] } else { 0 };

            (*state).edi = 0;
            (*state).ebp = 0;
            (*state).esp = state as u32;
            (*state).eip = entry_point;
            (*state).cs = 0x1B;
            (*state).eflags = 0x3202;
            (*state).ss = 0x23;
        }
    }
}

pub struct TaskManager {
    pub tasks: [Task; MAX_TASKS as usize],
    task_count: u32,
    current_task: i8,
}

pub static mut TASK_MANAGER: libk::mutex::Mutex<TaskManager> =
    libk::mutex::Mutex::new(TaskManager {
        tasks: [NULL_TASK; MAX_TASKS as usize],
        task_count: 0,
        current_task: -1,
    });

impl TaskManager {
    pub fn init(&mut self) {
        self.add_task(idle as u32, None);
    }

    pub fn add_task(&mut self, entry_point: u32, args: Option<&[u32]>) {
        if self.task_count < MAX_TASKS {
            let free_slot = self.get_free_slot();
            self.tasks[free_slot].init(entry_point, args);
            self.task_count += 1;
        }
    }

    pub fn add_user_task(&mut self, entry_point: u32, args: Option<&[u32]>) {
        if self.task_count < MAX_TASKS {
            let free_slot = self.get_free_slot();
            self.tasks[free_slot].init_u(entry_point, args);
            self.task_count += 1;
        }
    }

    pub fn schedule(&mut self, cpu_state: *mut CPUState) -> (*mut CPUState, u32) {
        if self.current_task >= 0 {
            self.tasks[self.current_task as usize].cpu_state_ptr = cpu_state as u32;

            if self.tasks[self.current_task as usize].state == TaskState::Zombie {
                unsafe {
                    (*(&raw mut crate::pmm::PADDR))
                        .dealloc(self.tasks[self.current_task as usize].stack);
                    (*(&raw mut crate::pmm::PADDR))
                        .dealloc(self.tasks[self.current_task as usize].kernel_stack);
                }

                self.tasks[self.current_task as usize] = NULL_TASK;
                self.task_count -= 1;
            }
        }

        self.current_task = self.get_next_task();
        if self.current_task < 0 {
            return (cpu_state, 0);
        }

        (
            self.tasks[self.current_task as usize].cpu_state_ptr as *mut CPUState,
            self.tasks[self.current_task as usize].kernel_stack,
        )
    }

    pub fn get_next_task(&self) -> i8 {
        let mut i = self.current_task + 1;
        while i < MAX_TASKS as i8 {
            let running = self.tasks[i as usize].state == TaskState::Ready;

            if running {
                return i;
            }

            i = (i + 1) % MAX_TASKS as i8;
        }

        -1
    }

    fn get_free_slot(&self) -> usize {
        for i in 0..MAX_TASKS as usize {
            if self.tasks[i].state == TaskState::Null {
                return i;
            }
        }

        panic!("No free slots available!");
    }
}

fn idle() {
    loop {
        unsafe { asm!("hlt") };
    }
}

pub fn exit() {
    unsafe {
        let t = (*(&raw mut TASK_MANAGER)).lock().current_task as usize;
        (*(&raw mut TASK_MANAGER)).lock().tasks[t].state = TaskState::Zombie;

        asm!("int 0x20");
    }
}

#[naked]
pub extern "C" fn timer() {
    unsafe {
        naked_asm!(
            "cli",
            "push eax",
            "push ebx",
            "push ecx",
            "push edx",
            "push esi",
            "push edi",
            "push ebp",
            "push esp",
            "call switch",
            "mov esp, eax",
            "pop ebp",
            "pop edi",
            "pop esi",
            "pop edx",
            "pop ecx",
            "pop ebx",
            "pop eax",
            "sti",
            "iretd",
        );
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn switch(esp: u32) -> u32 {
    unsafe {
        let new_esp = (*(&raw mut TASK_MANAGER))
            .lock()
            .schedule(esp as *mut CPUState);
        let k_stack = new_esp.1;
        let new_esp = new_esp.0 as u32;

        if k_stack != 0 {
            crate::set_tss(k_stack);
        }

        (*(&raw mut crate::pic::PICS)).end_interrupt(crate::exceptions::TIMER_INT);

        new_esp
    }
}
