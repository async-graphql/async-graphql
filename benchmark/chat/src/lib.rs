use async_graphql::*;

pub struct ChatData {
    pub id: String,
    pub created_at: String,
    pub title: String,
    pub caption: String,
    pub creator_user_id: String,
    pub state: String,
}

pub struct UserData {
    pub id: String,
    pub is_operator: bool,
    pub phone: u64,
    pub join_date: String,

    pub state: String,
}

pub struct ProfileData {
    pub first_name: String,
    pub last_name: String,
    pub city: Option<String>,
    pub job_title: Option<String>,
    pub email: String,
}

pub struct MessageData {
    pub id: String,
    pub user_id: String,
    pub timestamp: String,
    pub edited: bool,
    pub order: i32,
    pub message: String,
}

lazy_static::lazy_static! {
    pub static ref CHAT: ChatData = ChatData {
        id: "1".to_string(),
        created_at: "today".to_string(),
        title: "chat".to_string(),
        caption: "asdasd".to_string(),
        creator_user_id: "123".to_string(),
        state: "ACTIVE".to_string(),
    };

    pub static ref USER: UserData = UserData {
        id: "123".to_string(),
        is_operator: false,
        phone: 79_123_273_936,
        join_date: "today".to_string(),
        state: "ACTIVE".to_string(),
    };

    pub static ref PROFILE: ProfileData = ProfileData {
        first_name: "Ivan".to_string(),
        last_name: "Plesskih".to_string(),
        city: Some("Che".to_string()),
        job_title: Some("progr".to_string()),
        email: "asd@qwe.ru".to_string(),
    };

    pub static ref MESSAGE: MessageData = MessageData {
        id: "456".to_string(),
        user_id: "123".to_string(),
        timestamp: "today".to_string(),
        edited: false,
        order: 123,
        message: "Hello, world!".to_string(),
    };
}

pub struct Chat;

#[Object]
impl Chat {
    pub async fn id(&self) -> ID {
        ID::from(&CHAT.id)
    }

    pub async fn messages(&self) -> Vec<Message> {
        let mut res = vec![];
        for _ in 0..30 {
            res.push(Message);
        }
        res
    }

    pub async fn users(&self) -> Vec<User> {
        let mut res = vec![];
        for _ in 0..5 {
            res.push(User);
        }
        res
    }

    pub async fn creator(&self) -> User {
        User
    }

    #[field(name = "created_at")]
    pub async fn created_at(&self) -> &String {
        &CHAT.created_at
    }
    pub async fn title(&self) -> &String {
        &CHAT.title
    }
    pub async fn caption(&self) -> &String {
        &CHAT.caption
    }
    pub async fn state(&self) -> &String {
        &CHAT.state
    }
}

pub struct Message;

#[Object]
impl Message {
    pub async fn id(&self) -> ID {
        ID::from(&MESSAGE.id)
    }

    pub async fn user(&self) -> User {
        User
    }
    pub async fn timestamp(&self) -> &String {
        &MESSAGE.timestamp
    }
    pub async fn message(&self) -> &String {
        &MESSAGE.message
    }
    pub async fn order(&self) -> i32 {
        MESSAGE.order
    }
    pub async fn edited(&self) -> bool {
        MESSAGE.edited
    }
}

pub struct User;

#[Object]
impl User {
    pub async fn id(&self) -> ID {
        ID::from(&USER.id)
    }

    pub async fn profile(&self) -> Option<UserProfile> {
        Some(UserProfile)
    }

    #[field(name = "is_operator")]
    pub async fn is_operator(&self) -> bool {
        USER.is_operator
    }
    pub async fn phone(&self) -> String {
        USER.phone.to_string()
    }
    #[field(name = "join_date")]
    pub async fn join_date(&self) -> &String {
        &USER.join_date
    }
    pub async fn state(&self) -> &String {
        &USER.state
    }
}

pub struct UserProfile;

#[Object]
impl UserProfile {
    pub async fn email(&self) -> &String {
        &PROFILE.email
    }
    #[field(name = "first_name")]
    pub async fn first_name(&self) -> &String {
        &PROFILE.first_name
    }
    #[field(name = "last_name")]
    pub async fn last_name(&self) -> &String {
        &PROFILE.last_name
    }
    #[field(name = "job_title")]
    pub async fn job_title(&self) -> &Option<String> {
        &PROFILE.job_title
    }
    pub async fn city(&self) -> &Option<String> {
        &PROFILE.city
    }
}

pub struct Query;

#[Object]
impl Query {
    async fn chats(&self) -> Vec<Chat> {
        let mut res = vec![];
        for _ in 0..30 {
            res.push(Chat);
        }
        res
    }
}

lazy_static::lazy_static! {
    pub static ref S: Schema<Query, EmptyMutation, EmptySubscription> = Schema::new(Query, EmptyMutation, EmptySubscription);
}

pub const Q: &str = r#"
fragment User on User {
  id
  is_operator
  phone
  join_date
  state
  profile {
    email
    first_name
    last_name
    job_title
    city
  }
}

{
  chats {
    id
    created_at
    title
    caption
    state
    creator {
      ...User
    }
    messages {
      id
      timestamp
      edited
      message
      order
    }
    users {
      ...User
    }
  }
}
"#;
