use std::fmt::Debug;

#[derive(Debug)]
pub struct Set;
#[derive(Debug)]
pub struct Unset;

impl ToAssign for Set {}
impl ToAssign for Unset {}

impl Assigned for Set {}
impl NotAssigned for Unset {}

pub trait ToAssign: Debug {}
pub trait Assigned: ToAssign {}
pub trait NotAssigned: ToAssign {}
