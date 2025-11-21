#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {

    use stm32f4xx_hal::{
        gpio::{self, Edge, Input, Output, PushPull},
        pac::TIM1,
        prelude::*,
        rtc::{Rtc, Event},
        timer,
    };

    use defmt_rtt as _;

    // Resources shared between tasks
    #[shared]
    struct Shared {
        delayval: u32,
        rtc: Rtc,
    }

    // Local resources to specific tasks (cannot be shared)
    #[local]
    struct Local {
        button: gpio::PA0<Input>,
        led: gpio::PA13<Output<PushPull>>,
        delay: timer::DelayMs<TIM1>,
    }

    
    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {

        let mut dp = ctx.device;

        // Configure and obtain handle for delay abstraction
        // 1) Promote RCC structure to HAL to be able to configure clocks
        let mut rcc = dp.RCC.constrain();

        // Get CR registry to ensure we're able to make changes
        //let cr = dp.PWR.cr();
        //cr.modify(|_c, w| w.dbp().writable());

        //Configure RTC
        let mut rtc = Rtc::new(dp.RTC, &mut rcc, &mut dp.PWR);

        //Set date and time
        let _ = rtc.set_year(2023);
        let _ = rtc.set_month(11);
        let _ = rtc.set_day(25);
        let _ = rtc.set_hours(22);
        let _ = rtc.set_minutes(46);
        let _ = rtc.set_seconds(00);

        //Start listening to WAKE UP INTERRUPTS
        rtc.enable_wakeup(10.secs().into());
        rtc.listen(&mut dp.EXTI, Event::Wakeup);
        

        // 3) Create delay handle
        let delay = dp.TIM1.delay_ms(&mut rcc);

        // Configure the LED pin as a push pull ouput and obtain handle
        // On the Blackpill STM32F411CEU6 there is an on-board LED connected to pin PC13
        // 1) Promote the GPIOC PAC struct
        let gpioa = dp.GPIOA.split(&mut rcc);

        // 2) Configure PORTC OUTPUT Pins and Obtain Handle
        let led = gpioa.pa13.into_push_pull_output();
        // 2) Configure Pin and Obtain Handle
        let mut button = gpioa.pa0.into_pull_up_input();


        // Configure Button Pin for Interrupts
        // 1) Promote SYSCFG structure to HAL to be able to configure interrupts
        let mut syscfg = dp.SYSCFG.constrain(&mut rcc);
        // 2) Make button an interrupt source
        button.make_interrupt_source(&mut syscfg);
        // 3) Configure the interruption to be triggered on a rising edge
        button.trigger_on_edge(&mut dp.EXTI, Edge::Rising);
        // 4) Enable gpio interrupt for button
        button.enable_interrupt(&mut dp.EXTI);


        (
            // Initialization of shared resources
            Shared { delayval: 2000_u32, rtc},
            // Initialization of task local resources
            Local { button, led, delay},
        )
        
    }

    // Background task, runs whenever no other tasks are running
    #[idle(local = [led, delay], shared = [delayval])]
    fn idle(mut ctx: idle::Context) -> ! {
        let led = ctx.local.led;
        let delay = ctx.local.delay;
        loop {
            // Turn On LED
            led.set_high();
            // Obtain shared delay variable and delay
            delay.delay_ms(ctx.shared.delayval.lock(|del| *del));
            // Turn off LED
            led.set_low();
            // Obtain shared delay variable and delay
            delay.delay_ms(ctx.shared.delayval.lock(|del| *del));
        }
    }

    #[task(binds = EXTI0, local = [button], shared=[delayval, rtc])]
    fn gpio_interrupt_handler(mut ctx: gpio_interrupt_handler::Context) {

        ctx.shared.delayval.lock(|del| {
            *del = *del - 100_u32;
            if *del < 200_u32 {
                *del = 2000_u32;
            }
            *del
        });

        ctx.shared.rtc.lock(|rtc|{
            let current_time = rtc.get_datetime();
            
            defmt::info!("CURRENT TIME {:?}", current_time.as_hms());
            rtc.disable_wakeup();
        });
        
        ctx.local.button.clear_interrupt_pending_bit();

    }

    #[task(binds = RTC_WKUP, shared = [rtc])]
    fn rtc_wakeup(mut ctx: rtc_wakeup::Context) {
        defmt::warn!("RTC INTERRUPT!!!!");
        ctx.shared.rtc.lock(|rtc|{
            let current_time = rtc.get_datetime();
            rtc.clear_interrupt(Event::Wakeup);
            defmt::info!("Current time {:?}", current_time.as_hms() );
        });
        // Your RTC wakeup interrupt handling code here        
    }
}
