use std::sync::atomic::{AtomicU32, Ordering};

use crate::{Control, Filter, Frame, Source, StridedMut};

/// Scales amplitude by a dynamically-adjustable factor
pub struct Gain<T: ?Sized> {
    gain: AtomicU32,
    inner: T,
}

impl<T> Gain<T> {
    /// Apply dynamic gain to `source`
    pub fn new(source: T) -> Self {
        Self {
            gain: AtomicU32::new(1.0f32.to_bits()),
            inner: source,
        }
    }
}

impl<T: Source> Source for Gain<T>
where
    T::Frame: Frame,
{
    type Frame = T::Frame;

    fn sample(&self, offset: f32, sample_length: f32, mut out: StridedMut<'_, Self::Frame>) {
        self.inner.sample(offset, sample_length, out.borrow());
        // Should we blend from the previous value?
        let gain = f32::from_bits(self.gain.load(Ordering::Relaxed));
        for x in &mut out {
            *x = x.scale(gain);
        }
    }

    fn advance(&self, dt: f32) {
        self.inner.advance(dt);
    }

    fn remaining(&self) -> f32 {
        self.inner.remaining()
    }
}

impl<T> Filter for Gain<T> {
    type Inner = T;
    fn inner(&self) -> &T {
        &self.inner
    }
}

impl<T> Control<'_, Gain<T>> {
    /// Get the current gain
    pub fn gain(&self) -> f32 {
        unsafe { f32::from_bits((*self.get()).gain.load(Ordering::Relaxed)) }
    }

    /// Adjust the gain
    pub fn set_gain(&mut self, factor: f32) {
        unsafe {
            (*self.get())
                .gain
                .store(factor.to_bits(), Ordering::Relaxed);
        }
    }
}