use std::fmt::Debug;

#[derive(Debug)]
pub struct Yes;
#[derive(Debug)]
pub struct No;

impl ToAssign for Yes {}
impl ToAssign for No {}

impl Assigned for Yes {}
impl NotAssigned for No {}

pub trait ToAssign: Debug {}
pub trait Assigned: ToAssign {}
pub trait NotAssigned: ToAssign {}
