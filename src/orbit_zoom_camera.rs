#![allow(dead_code)]

//! A 3dsMax / Blender style camera that orbits about a target position

use vecmath::{ Vector3, vec3_add, vec3_scale };
use vecmath::traits::Float;

use quaternion;
use quaternion::Quaternion;

use input::{ GenericEvent, Key, MouseButton };
use input::Button::{ Keyboard, Mouse };

use Camera;

bitflags!(
    pub struct Mode: u8 {
        const ORBIT_BUTTON = 0b00000001;
        const ZOOM_BUTTON  = 0b00000010;
        const PAN_BUTTON   = 0b00000100;
        const ORBIT_MOD    = 0b00001000;
        const ZOOM_MOD     = 0b00010000;
        const PAN_MOD      = 0b00100000;
    }
);

/// Specifies key bindings and speed modifiers for OrbitZoomCamera
pub struct OrbitZoomCameraSettings<T=f32> {

    /// Which mouse button to press to orbit with mouse
    pub orbit_button: MouseButton,

    /// Which mouse button to press to zoom with mouse
    pub zoom_button: MouseButton,

    /// Which mouse button to press to pan with mouse
    pub pan_button: MouseButton,

    /// Which key to press to orbit with mouse (if any)
    pub orbit_mod: Option<Key>,

    /// Which key to press to zoom with mouse (if any)
    pub zoom_mod: Option<Key>,

    /// Which key to press to pan with mouse (if any)
    pub pan_mod: Option<Key>,

    /// What does scrolling control (the given bits are automatically set when scrolling)
    pub scroll_mode: Mode,

    /// Modifier for orbiting speed (arbitrary unit)
    pub orbit_speed: T,

    /// Modifier for pitch speed relative to orbiting speed (arbitrary unit).
    /// To reverse pitch direction, set this to -1.
    pub pitch_speed: T,

    /// Modifier for panning speed (arbitrary unit)
    pub pan_speed: T,

    /// Modifier for zoom speed (arbitrary unit)
    pub zoom_speed: T,
}

impl<T: Float> OrbitZoomCameraSettings<T> {

    /// Clicking and dragging OR two-finger scrolling will orbit camera,
    /// with LShift as pan modifer and LCtrl as zoom modifier
    pub fn default() -> OrbitZoomCameraSettings<T> {
        OrbitZoomCameraSettings {
            orbit_button : MouseButton::Left,
            zoom_button : MouseButton::Right,
            pan_button : MouseButton::Left,
            orbit_mod : None,
            zoom_mod : None,
            pan_mod : Some(Key::LShift),
            scroll_mode: ZOOM_BUTTON,
            orbit_speed: T::from_f32(0.05),
            pitch_speed: T::from_f32(0.1),
            pan_speed: T::from_f32(0.1),
            zoom_speed: T::from_f32(0.1),
        }
    }

    /// Set the button for orbiting
    pub fn orbit_button(self, button: MouseButton) -> OrbitZoomCameraSettings<T> {
        OrbitZoomCameraSettings {
            orbit_button: button,
            .. self
        }
    }

    /// Set the button for zooming
    pub fn zoom_button(self, button: MouseButton) -> OrbitZoomCameraSettings<T> {
        OrbitZoomCameraSettings {
            zoom_button: button,
            .. self
        }
    }

    /// Set the button for panning
    pub fn pan_button(self, button: MouseButton) -> OrbitZoomCameraSettings<T> {
        OrbitZoomCameraSettings {
            pan_button: button,
            .. self
        }
    }

    /// Set what scrolling does by default
    pub fn scroll_mode(self, mode: Mode) -> OrbitZoomCameraSettings<T> {
        OrbitZoomCameraSettings {
            scroll_mode: mode,
            .. self
        }
    }

    /// Set the orbit speed modifier
    pub fn orbit_speed(self, s: T) -> OrbitZoomCameraSettings<T> {
        OrbitZoomCameraSettings {
            orbit_speed: s,
            .. self
        }
    }

    /// Set the pitch speed modifier
    pub fn pitch_speed(self, s: T) -> OrbitZoomCameraSettings<T> {
        OrbitZoomCameraSettings {
            pitch_speed: s,
            .. self
        }
    }

    /// Set the pan speed modifier
    pub fn pan_speed(self, s: T) -> OrbitZoomCameraSettings<T> {
        OrbitZoomCameraSettings {
            pan_speed: s,
            .. self
        }
    }

    /// Set the zoom speed modifier
    pub fn zoom_speed(self, s: T) -> OrbitZoomCameraSettings<T> {
        OrbitZoomCameraSettings {
            zoom_speed: s,
            .. self
        }
    }
}

/// A 3dsMax / Blender-style camera that orbits around a target point
pub struct OrbitZoomCamera<T=f32> {

    /// origin of camera rotation
    pub target: Vector3<T>,

    /// Rotation of camera
    pub rotation: Quaternion<T>,

    /// Pitch up/down from target
    pub pitch: T,

    /// Yaw left/right from target
    pub yaw: T,

    /// Camera distance from target
    pub distance: T,

    /// Lower distance limit
    /// Set this to near clipping distance
    pub distance_near_limit: T,

    /// Upper distance limit
    /// Set this to far clipping distance
    pub distance_far_limit: T,

    /// Settings for the camera
    pub settings: OrbitZoomCameraSettings<T>,

    /// Current camera control mode activated
    mode: Mode,
}


impl<T: Float>
OrbitZoomCamera<T> {
    /// Create a new OrbitZoomCamera targeting the given coordinates
    pub fn new(target: [T; 3], settings: OrbitZoomCameraSettings<T>) -> OrbitZoomCamera<T> {

        // If there is no modifier button, assume it's as if the button is always pressed
        let mut mode = Mode::empty();
        if settings.orbit_mod == None {
            mode |= ORBIT_MOD;
        }
        if settings.zoom_mod == None {
            mode |= ZOOM_MOD;
        }
        if settings.pan_mod == None {
            mode |= PAN_MOD;
        }

        OrbitZoomCamera {
            target: target,
            rotation: quaternion::id(),
            distance: T::from_f32(10.0),
            distance_near_limit: T::from_f32(0.1),
            distance_far_limit: T::from_f32(1000.0),
            pitch: T::zero(),
            yaw: T::zero(),
            mode,
            settings,
        }
    }

    /// Return a Camera for the current OrbitZoomCamera configuration
    pub fn camera(&self, _dt: f64) -> Camera<T> {
        let target_to_camera = quaternion::rotate_vector(
            self.rotation,
            [T::zero(), T::zero(), self.distance]
        );
        let mut camera = Camera::new(vec3_add(self.target, target_to_camera));
        camera.set_rotation(self.rotation);
        camera
    }

    fn rotation_from_yaw_and_pitch(yaw: T, pitch: T) -> Quaternion<T> {
        let _1 = T::one();
        let _0 = T::zero();
        quaternion::mul(
            quaternion::axis_angle([_0, _1, _0], yaw),
            quaternion::axis_angle([_1, _0, _0], pitch)
            )
    }

    /// Initialize the camera configuration, such that next call to camera() gives the correct
    /// camera rotation
    pub fn init(&mut self) {
        self.rotation = Self::rotation_from_yaw_and_pitch(self.yaw, self.pitch);
    }

    fn is_orbit(&self) -> bool {
        self.mode.contains(ORBIT_BUTTON | ORBIT_MOD)
    }

    fn is_zoom(&self) -> bool {
        self.mode.contains(ZOOM_BUTTON | ZOOM_MOD)
    }

    fn is_pan(&self) -> bool {
        self.mode.contains(PAN_BUTTON | PAN_MOD)
    }

    /// Orbit the camera using the given horizontal and vertical params,
    /// or zoom or pan if the appropriate modifier keys are pressed
    pub fn control_camera(&mut self, dx: T, dy: T) {

        let _1 = T::one();
        let _0 = T::zero();

        if self.is_pan() {

            // Pan target position along plane normal to camera direction
            let dx = dx * self.settings.pan_speed*self.distance;
            let dy = dy * self.settings.pan_speed*self.distance;

            let right = quaternion::rotate_vector(self.rotation, [_1, _0, _0]);
            let up = quaternion::rotate_vector(self.rotation, [_0, _1, _0]);
            self.target = vec3_add(
                vec3_add(self.target, vec3_scale(up, dy)),
                vec3_scale(right,dx)
            );

        } else if self.is_zoom() {

            // Zoom to / from target
            let new_dist = self.distance + dy * self.settings.zoom_speed*self.distance;
            self.distance =
                if new_dist > self.distance_far_limit {
                    self.distance_far_limit
                } else if new_dist < self.distance_near_limit {
                    self.distance_near_limit
                } else {
                    new_dist
                }

        } else if self.is_orbit() {

            // Orbit around target
            let dx = dx * self.settings.orbit_speed;
            let dy = dy * self.settings.orbit_speed;

            self.yaw = self.yaw + dx;
            self.pitch = self.pitch + dy*self.settings.pitch_speed;
            self.rotation = Self::rotation_from_yaw_and_pitch(self.yaw, self.pitch);
        }
    }

    fn mod_key_pressed(&self) -> bool {
        let mut is_pressed = false;
        if let Some(_) = self.settings.orbit_mod {
            if self.mode.contains(ORBIT_MOD) {
                is_pressed = true;
            }
        } else if let Some(_) = self.settings.zoom_mod {
            if self.mode.contains(ZOOM_MOD) {
                is_pressed = true;
            }
        } else if let Some(_) = self.settings.pan_mod {
            if self.mode.contains(PAN_MOD) {
                is_pressed = true;
            }
        }
        is_pressed
    }


    /// Respond to scroll and key press/release events
    pub fn event<E: GenericEvent>(&mut self, e: &E) {
        //e.touch(|args| {
        //    println!("args = {:?}", args);
        //});
        e.mouse_scroll(|dx, dy| {
            let dx = T::from_f64(dx);
            let dy = T::from_f64(dy);

            // if a mod key is pressed, override the default scroll mode
            let mut restore = false;
            if !self.mod_key_pressed() {
                restore = !self.mode.contains(self.settings.scroll_mode);
                self.mode.insert(self.settings.scroll_mode);
            }

            self.control_camera(dx, dy);
            if restore {
                self.mode.remove(self.settings.scroll_mode);
            }
        });

        e.mouse_relative(|dx, dy| {
            let dx = T::from_f64(dx);
            let dy = T::from_f64(dy);
            self.control_camera(-dx, dy);
        });

        e.press(|x| {
            if x == Mouse(self.settings.orbit_button) { self.mode.insert(ORBIT_BUTTON); }
            if x == Mouse(self.settings.pan_button) { self.mode.insert(PAN_BUTTON); }
            if x == Mouse(self.settings.zoom_button) { self.mode.insert(ZOOM_BUTTON); }
            if Some(x) == self.settings.orbit_mod.map(|a| Keyboard(a)) { self.mode.insert(ORBIT_MOD); }
            if Some(x) == self.settings.pan_mod.map(|a| Keyboard(a)) { self.mode.insert(PAN_MOD); }
            if Some(x) == self.settings.zoom_mod.map(|a| Keyboard(a)) { self.mode.insert(ZOOM_MOD); }
        });

        e.release(|x| {
            if x == Mouse(self.settings.orbit_button) { self.mode.remove(ORBIT_BUTTON); }
            if x == Mouse(self.settings.pan_button) { self.mode.remove(PAN_BUTTON); }
            if x == Mouse(self.settings.zoom_button) { self.mode.remove(ZOOM_BUTTON); }
            if Some(x) == self.settings.orbit_mod.map(|a| Keyboard(a)) { self.mode.remove(ORBIT_MOD); }
            if Some(x) == self.settings.pan_mod.map(|a| Keyboard(a)) { self.mode.remove(PAN_MOD); }
            if Some(x) == self.settings.zoom_mod.map(|a| Keyboard(a)) { self.mode.remove(ZOOM_MOD); }
        });
    }

}
