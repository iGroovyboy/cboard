const clamp = (value, min, max) => {
  return Math.max(min, Math.min(value, max));
};

const createSlideButton = (id, selector, slide) => {
  const button = document.createElement("button");
  button.classList.add(selector);
  if (id === 0) {
    button.classList.add("active");
  }
  button.addEventListener("click", () => {
    document.querySelectorAll(`.${selector}`).forEach((element) => {
      element.classList.remove("active");
    });
    button.classList.add("active");
    slide.parentElement.scrollLeft = slide.clientWidth * id;
  });

  return button;
};

const createSlider = (slideSelector, sliderControls, slideBtnSelector) => {
  const slides = document.querySelectorAll(slideSelector);
  if (!slides.length) {
    return;
  }

  slides.forEach((slide, id) => {
    const button = createSlideButton(id, slideBtnSelector, slide);
    document.querySelector(sliderControls)?.appendChild(button);
  });
};

createSlider(".slide", ".slider-controls", "slider-btn");
