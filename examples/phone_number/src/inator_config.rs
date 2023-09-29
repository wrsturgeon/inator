pub mod phone_number {
    use core::mem::MaybeUninit;

    pub struct Output {
        digits: [MaybeUninit<u8>; 10],
        progress: u8,
    }

    impl From<Output> for crate::PhoneNumber {
        #[inline]
        fn from(value: Output) -> Self {
            let d = value.digits;
            Self {
                area_code: unsafe { [d[0].assume_init(), d[1].assume_init(), d[2].assume_init()] },
                number: unsafe {
                    [
                        d[3].assume_init(),
                        d[4].assume_init(),
                        d[5].assume_init(),
                        d[6].assume_init(),
                        d[7].assume_init(),
                        d[8].assume_init(),
                        d[9].assume_init(),
                    ]
                },
            }
        }
    }

    pub fn initial() -> Output {
        Output {
            digits: unsafe { MaybeUninit::uninit().assume_init() },
            progress: 0,
        }
    }

    pub fn digit(mut number: Output, token: char) -> Output {
        number.digits[number.progress as usize].write(token as u8 - b'0');
        number.progress += 1;
        number
    }
}
