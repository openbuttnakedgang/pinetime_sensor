use embedded_hal::{
    digital::v2::OutputPin,
    blocking::delay::DelayUs,
    blocking::spi
};
use embedded_graphics::{
    style,
    prelude::*,
    pixelcolor,
    primitives,
    fonts
};

const BACKGROUND_COLOR: pixelcolor::Rgb565 = pixelcolor::Rgb565::WHITE;
const AXES_COLOR:       pixelcolor::Rgb565 = pixelcolor::Rgb565::BLACK;
const LINE_COLOR:       pixelcolor::Rgb565 = pixelcolor::Rgb565::RED;

const MARGIN: u16 = 10;
pub const LCD_W: u16 = 240;
pub const LCD_H: u16 = 240;

pub struct SPIDriver<RST, SPI, DC, DELAY> 
where
    SPI: spi::Write<u8>,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayUs<u32>,
{
    pub display_driver: st7789::ST7789::<SPI, DC, RST, DELAY>,
    plot_values: [Point; LCD_W as usize]
}

impl<RST, SPI, DC, DELAY, E> SPIDriver<RST, SPI, DC, DELAY> 
where 
    SPI: spi::Write<u8>,
    DC: OutputPin<Error = E>,
    RST: OutputPin<Error = E>,
    DELAY: DelayUs<u32>,
    E: core::fmt::Debug
{
    pub fn new(display_driver: st7789::ST7789<SPI, DC, RST, DELAY>) -> Self {
        SPIDriver::<RST, SPI, DC, DELAY> { 
            display_driver, 
            plot_values: [Point::default(); LCD_W as usize]
        }
    }
    
    pub fn init(&mut self) -> Result<(), st7789::Error<SPI::Error, DC::Error, RST::Error>> {
        self.display_driver.init()?;       
        self.display_driver.set_orientation(&st7789::Orientation::Portrait)?;
        Ok(())
    }

    pub fn draw_text(&mut self) -> Result<(), st7789::Error<SPI::Error, DC::Error, RST::Error>> {
        // Draw something onto the LCD
        let backdrop_style = style::PrimitiveStyleBuilder::new()
            .fill_color(pixelcolor::Rgb565::RED)
            .build();

        primitives::rectangle::Rectangle::new(
            Point::new(0, 0), 
            Point::new(LCD_W as i32, LCD_H as i32)
        )
            .into_styled(backdrop_style)
            .draw(&mut self.display_driver)?;

        // Choose text style
        let text_style = style::TextStyleBuilder::new(fonts::Font12x16)
            .text_color(pixelcolor::Rgb565::WHITE)
            .background_color(AXES_COLOR);

        // Draw text
        fonts::Text::new("HRS data ...", Point::new(10, 10))
            .into_styled(text_style.build())
            .draw(&mut self.display_driver)?;

        // Draw text
        fonts::Text::new("20%", Point::new(10, 10 + 16 + MARGIN as i32))
            .into_styled(text_style.build())
            .draw(&mut self.display_driver)?;

        Ok(())
    }

    pub fn draw_axes(&mut self) -> Result<(), st7789::Error<SPI::Error, DC::Error, RST::Error>> {
        // background
        let backdrop_style = style::PrimitiveStyleBuilder::new()
            .fill_color(BACKGROUND_COLOR)
            .build();

        primitives::rectangle::Rectangle::new(
            Point::new(0, 0), 
            Point::new(LCD_W as i32, LCD_H as i32)
        )
            .into_styled(backdrop_style)
            .draw(&mut self.display_driver)?;

        let line_style = style::PrimitiveStyleBuilder::new()
            .stroke_color(AXES_COLOR)
            .stroke_width(1)
            .build();
            
        // X axis
        let ox1 = Point::new(0,             LCD_H as i32 / 2);
        let ox2 = Point::new(LCD_W as i32,  LCD_H as i32 / 2 + 1);
        primitives::line::Line::new(ox1, ox2)
            .into_styled(line_style)
            .draw(&mut self.display_driver)?;

        // Y axis
        let oy1 = Point::new(LCD_H as i32 / 2,      0);
        let oy2 = Point::new(LCD_H as i32 / 2 + 1,  LCD_H as i32);
        primitives::line::Line::new(oy1, oy2)
            .into_styled(line_style)
            .draw(&mut self.display_driver)?;

        Ok(())
    }

    pub fn count_sin(&mut self) {
        let x_min = -(LCD_W as i32) / 2;
        let x_max = (LCD_W as i32) / 2;
        let mut x = x_min-1;
        let mut y: i32;

        let div = 20_f32;
        let mul = 100_f32;

        for p in self.plot_values.iter_mut() {
            x += 1;
            y = Self::sin(x, div, mul);
            p.x = x;
            p.y = y;
        }
    }

    // y = mul * sin(x / div)
    fn sin(x: i32, div: f32, mul: f32) -> i32 {
        let mut y: f32 = 0_f32;
        let x = x as f32;
    
        let mut sign: f32 = 1_f32;
        let mut dx: f32 = x / div;
    
        let mut den_factor: f32 = 1_f32;
    
        loop {
            y += sign * dx; 
    
            // calculate next addition            
            dx *= x / div * x / div;
    
            den_factor += 1_f32;
            dx /= den_factor;
            den_factor += 1_f32;
            dx /= den_factor;
            //println!("dx: '{}'", dx);
    
            if dx < 0.1_f32 && dx > -0.1_f32 { break; }
    
            sign *= -1_f32;
        }
    
        (y * mul) as i32
    }

    pub fn draw_sin(&mut self) -> Result<(), st7789::Error<SPI::Error, DC::Error, RST::Error>> {
        let backdrop_style = style::PrimitiveStyleBuilder::new()
            .stroke_color(LINE_COLOR)
            .stroke_width(1)
            .build();

        let mut p1: Point = self.plot_values[0];
        for p2 in &(self.plot_values[1..]) {
            primitives::line::Line::new(
                    Self::transform(p1), 
                    Self::transform(*p2)
                )
                .into_styled(backdrop_style)
                .draw(&mut self.display_driver)?;

            p1 = *p2;
        }

        Ok(())
    }

    pub fn clear_sin(&mut self) -> Result<(), st7789::Error<SPI::Error, DC::Error, RST::Error>> {
        let backdrop_style = style::PrimitiveStyleBuilder::new()
            .stroke_color(BACKGROUND_COLOR)
            .stroke_width(1)
            .build();

        let mut p1: Point = self.plot_values[0];
        for p2 in &(self.plot_values[1..]) {
            primitives::line::Line::new(
                    Self::transform(p1), 
                    Self::transform(*p2)
                )
                .into_styled(backdrop_style)
                .draw(&mut self.display_driver)?;

            p1 = *p2;
        }

        Ok(())
    }

    fn transform(p: Point) -> Point {
        Point::new(
            Self::clamp(p.y + LCD_H as i32 / 2, 0, LCD_W),
            Self::clamp(p.x + LCD_W as i32 / 2, 0, LCD_H)
        )
    }

    fn clamp(x: i32, min: u16, max: u16) -> i32 {
        let x_u16 = x as u16;
        if x_u16 > max { max as i32 }
        else if x_u16 < min { min as i32 }
        else { x as i32 }
    }
    
    pub fn rotate_sin(&mut self) {
        let y = self.plot_values[0].y;
        for i in 1..self.plot_values.len() {
            self.plot_values[i-1].y = self.plot_values[i].y;
        }
        (self.plot_values.last_mut().unwrap()).y = y;
    }
}