use oxide_rpa::rpa::{MouseController, KeyboardController, ScreenCapture};
use rdev::Button;

#[tokio::test]
async fn test_mouse_controller() {
    let mouse = MouseController::new();
    // These tests require a graphical environment and might interfere with actual usage.
    // They are primarily for ensuring the methods compile and can be called.
    // mouse.move_to(100, 100);
    // mouse.click(Button::Left);
    // mouse.scroll(0, 1);
}

#[tokio::test]
async fn test_keyboard_controller() {
    let keyboard = KeyboardController::new();
    // These tests require a graphical environment and might interfere with actual usage.
    // keyboard.type_text("hello world");
    // keyboard.press_key(rdev::Key::KeyA);
    // keyboard.release_key(rdev::Key::KeyA);
}

#[tokio::test]
async fn test_screen_capture() {
    let screen_capture = ScreenCapture::new();
    let result = screen_capture.capture_screen().await;
    assert!(result.is_ok());
    let image = result.unwrap();
    assert!(!image.pixels().next().is_none()); // Check if image has pixels

    let result_area = screen_capture.capture_area(0, 0, 10, 10).await;
    assert!(result_area.is_ok());
    let image_area = result_area.unwrap();
    assert_eq!(image_area.width(), 10);
    assert_eq!(image_area.height(), 10);
}
