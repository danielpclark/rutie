use rubysys::value::ValueType;

pub const FL_WB_PROTECTED: isize  = (1<<5);
pub const FL_PROMOTED0   : isize  = (1<<5);
pub const FL_PROMOTED1   : isize  = (1<<6);
pub const FL_PROMOTED    : isize  = FL_PROMOTED0|FL_PROMOTED1;
pub const FL_FINALIZE    : isize  = (1<<7);
pub const FL_TAINT       : isize  = (1<<8);
pub const FL_UNTRUSTED   : isize  = FL_TAINT;
pub const FL_EXIVAR      : isize  = (1<<10);
pub const FL_FREEZE      : isize  = (1<<11);

pub const FL_USHIFT      : isize = 12;

pub const FL_USER_0      : isize = 1 << (FL_USHIFT + 0);
pub const FL_USER_1      : isize = 1 << (FL_USHIFT + 1);
pub const FL_USER_2      : isize = 1 << (FL_USHIFT + 2);
pub const FL_USER_3      : isize = 1 << (FL_USHIFT + 3);
pub const FL_USER_4      : isize = 1 << (FL_USHIFT + 4);
pub const FL_USER_5      : isize = 1 << (FL_USHIFT + 5);
pub const FL_USER_6      : isize = 1 << (FL_USHIFT + 6);
pub const FL_USER_7      : isize = 1 << (FL_USHIFT + 7);
pub const FL_USER_8      : isize = 1 << (FL_USHIFT + 8);
pub const FL_USER_9      : isize = 1 << (FL_USHIFT + 9);
pub const FL_USER_10     : isize = 1 << (FL_USHIFT + 10);
pub const FL_USER_11     : isize = 1 << (FL_USHIFT + 11);
pub const FL_USER_12     : isize = 1 << (FL_USHIFT + 12);
pub const FL_USER_13     : isize = 1 << (FL_USHIFT + 13);
pub const FL_USER_14     : isize = 1 << (FL_USHIFT + 14);
pub const FL_USER_15     : isize = 1 << (FL_USHIFT + 15);
pub const FL_USER_16     : isize = 1 << (FL_USHIFT + 16);
pub const FL_USER_17     : isize = 1 << (FL_USHIFT + 17);
pub const FL_USER_18     : isize = 1 << (FL_USHIFT + 18);


pub const ELTS_SHARED : isize = FL_USER_2;
pub const FL_DUPPED   : isize = (ValueType::Mask as isize|FL_EXIVAR|FL_TAINT);
pub const FL_SINGLETON: isize = FL_USER_0;
