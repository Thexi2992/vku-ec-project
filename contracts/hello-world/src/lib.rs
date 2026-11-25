#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String};

// Danh sách lỗi của contract (mỗi lỗi một số, bắt đầu từ 1)
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    EventExists = 1,
    EventNotFound = 2,
    InvalidMaxTickets = 3,
    SoldOut = 4,
    AlreadyRegistered = 5,
    NotOrganizer = 6,
    NoTicket = 7,
    AlreadyCheckedIn = 8,
}

// Thông tin một sự kiện
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub name: String,
    pub organizer: Address,
    pub max_tickets: u32,
    pub issued: u32,
    pub created_at: u64,
}

// Trạng thái vé của một người trong một sự kiện
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TicketStatus {
    NotRegistered,
    Registered,
    CheckedIn,
}

// Khoá lưu trữ
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Event(u32),
    Ticket(u32, Address),
}

// Hàm phụ: đọc sự kiện từ storage, báo lỗi nếu không tồn tại
fn load_event(env: &Env, event_id: u32) -> Result<Event, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Event(event_id))
        .ok_or(Error::EventNotFound)
}

#[contract]
pub struct ClubTicket;

#[contractimpl]
impl ClubTicket {
    // Ban tổ chức tạo sự kiện mới với tên và số vé tối đa
    pub fn create_event(
        env: Env,
        organizer: Address,
        event_id: u32,
        name: String,
        max_tickets: u32,
    ) -> Result<(), Error> {
        organizer.require_auth();
        if max_tickets == 0 {
            return Err(Error::InvalidMaxTickets);
        }
        let key = DataKey::Event(event_id);
        if env.storage().persistent().has(&key) {
            return Err(Error::EventExists);
        }
        let event = Event {
            name,
            organizer,
            max_tickets,
            issued: 0,
            created_at: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&key, &event);
        Ok(())
    }

    // Sinh viên tự đăng ký lấy một vé cho sự kiện
    pub fn register(env: Env, student: Address, event_id: u32) -> Result<(), Error> {
        student.require_auth();
        let mut event = load_event(&env, event_id)?;
        let ticket_key = DataKey::Ticket(event_id, student);
        if env.storage().persistent().has(&ticket_key) {
            return Err(Error::AlreadyRegistered);
        }
        if event.issued >= event.max_tickets {
            return Err(Error::SoldOut);
        }
        event.issued += 1;
        env.storage().persistent().set(&ticket_key, &TicketStatus::Registered);
        env.storage().persistent().set(&DataKey::Event(event_id), &event);
        Ok(())
    }

    // Ban tổ chức check-in một sinh viên đã có vé (không check-in hai lần)
    pub fn check_in(
        env: Env,
        organizer: Address,
        student: Address,
        event_id: u32,
    ) -> Result<(), Error> {
        organizer.require_auth();
        let event = load_event(&env, event_id)?;
        if event.organizer != organizer {
            return Err(Error::NotOrganizer);
        }
        let ticket_key = DataKey::Ticket(event_id, student);
        let status: TicketStatus = env
            .storage()
            .persistent()
            .get(&ticket_key)
            .ok_or(Error::NoTicket)?;
        if status == TicketStatus::CheckedIn {
            return Err(Error::AlreadyCheckedIn);
        }
        env.storage().persistent().set(&ticket_key, &TicketStatus::CheckedIn);
        Ok(())
    }

    // Xem toàn bộ thông tin của một sự kiện
    pub fn get_event(env: Env, event_id: u32) -> Result<Event, Error> {
        load_event(&env, event_id)
    }

    // Xem số vé đã phát của sự kiện
    pub fn tickets_issued(env: Env, event_id: u32) -> Result<u32, Error> {
        Ok(load_event(&env, event_id)?.issued)
    }

    // Xem số vé còn lại của sự kiện
    pub fn tickets_left(env: Env, event_id: u32) -> Result<u32, Error> {
        let event = load_event(&env, event_id)?;
        Ok(event.max_tickets - event.issued)
    }

    // Kiểm tra một người chưa có vé, đã có vé hay đã check-in
    pub fn get_ticket_status(
        env: Env,
        event_id: u32,
        student: Address,
    ) -> Result<TicketStatus, Error> {
        load_event(&env, event_id)?;
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::Ticket(event_id, student))
            .unwrap_or(TicketStatus::NotRegistered))
    }
}