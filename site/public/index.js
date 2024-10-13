document.addEventListener('DOMContentLoaded', () => {
});

const winHeight = window.innerHeight;
const sections = document.querySelectorAll('section');
let currentSection = 0;  // Index of the current section
let isAnimating = false;

const scrollToSection = (index) => {     
    isAnimating = true;
    window.scrollTo({behavior: "smooth", top: index * winHeight});
    setTimeout(() => { isAnimating = false; }, 400)
};

const clamp = (value, min, max) => {
    return Math.max(min, Math.min(value, max));
}
  
const scrollToNextSection = (index) => {
    scrollToSection(clamp(index, 0, sections.length));
}

document.addEventListener('wheel', (event) => {
    event.preventDefault();

    if (isAnimating) {
        return
    }; 

    if (event.deltaY > 0 && currentSection < sections.length - 1) {
        currentSection++;
    } else if (event.deltaY < 0 && currentSection > 0) {
        currentSection--;
    }
    
    scrollToSection(currentSection);
}, { passive: false });

// --------- slider

const createSlideButton = (id, selector, slide) => {
    const button = document.createElement('button');
    button.classList.add(selector);
    if (id === 0) {
        button.classList.add('active');
    }
    button.addEventListener('click', () => {
        document.querySelectorAll(`.${selector}`).forEach(element => {
            element.classList.remove('active');
        });
        button.classList.add('active');
        slide.parentElement.scrollLeft = slide.clientWidth * id
    });

    return button;
}

const createSlider = (slideSelector, sliderControls, slideBtnSelector) => { 
    const slides = document.querySelectorAll(slideSelector);
    if (!slides.length) {
        return;
    }

    slides.forEach((slide, id)=> {
        const button = createSlideButton(id, slideBtnSelector, slide);
        document.querySelector(sliderControls)?.appendChild(button);
    });
}

createSlider('.slide', '.slider-controls', 'slider-btn');
