let app = null;
let isRunning = null;
let animationFrameId = null;

if (window.wasmBindings) {
    init();
} else {
    window.addEventListener('TrunkApplicationStarted', () => {
        init();
    });
}

function init() {
    try {
        const App = window.wasmBindings.App;
        app = new App();
        console.log('Emulator initialized!');

        setupFileInput();
        setupKeyboard();
        setupControlButtons();
    } catch (e) {
        console.error('Failed to init:', e);
    }
}

function setupFileInput() {
    const fileInput = document.getElementById('rom-input');
    if (!fileInput) {
        return;
    }
    fileInput.addEventListener('change', (event) => {
        const file = event.target.files[0];
        if (!file) return;
        const reader = new FileReader();
        reader.onload = (e) => {
            const romData = new Uint8Array(e.target.result);
            if (app) {
                try {
                    app.load_rom(romData);
                    console.log('ROM loaded:', file.name);
                    app.reset();
                    startLoop();
                    updateToggleButton();
                } catch (err) {
                    console.error('Failed to load ROM:', err);
                }
            }
        };
        reader.readAsArrayBuffer(file);
    });
}

function setupKeyboard() {
    const {Button} = window.wasmBindings;
    const keyMap = {
        'ArrowUp': Button.Up,
        'ArrowDown': Button.Down,
        'ArrowLeft': Button.Left,
        'ArrowRight': Button.Right,
        'd': Button.A,
        'f': Button.B,
        'c': Button.Start,
        'v': Button.Select
    };
    document.addEventListener('keydown', (event) => {
        const button = keyMap[event.key];
        if (button !== undefined) {
            event.preventDefault();
            app.set_button(button, true);
        }
    });
    document.addEventListener('keyup', (event) => {
        const button = keyMap[event.key];
        if (button !== undefined) {
            event.preventDefault();
            app.set_button(button, false);
        }
    });
}

function setupControlButtons() {
    const toggleBtn = document.getElementById('toggle-run-btn');
    const resetBtn = document.getElementById('reset-btn');

    if (toggleBtn) {
        toggleBtn.addEventListener('click', () => {
            if (isRunning) {
                stopLoop();
            } else {
                startLoop();
            }
            updateToggleButton();
        });
    }

    if (resetBtn) {
        resetBtn.addEventListener('click', resetApp);
        updateToggleButton();
    }

    updateToggleButton();
}

function updateToggleButton() {
    const btn = document.getElementById('toggle-run-btn');
    if (btn) {
        btn.textContent = isRunning ? 'Stop' : 'Run';
    }
}

function startLoop() {
    console.log("start")
    if (animationFrameId) {
        return;
    }
    isRunning = true;

    function loop() {
        if (isRunning && app) {
            app.step_frame_and_render();
        }
        animationFrameId = requestAnimationFrame(loop);
    }

    animationFrameId = requestAnimationFrame(loop);
}

function stopLoop() {
    console.log("stop")
    isRunning = false;
    if (animationFrameId) {
        cancelAnimationFrame(animationFrameId);
        animationFrameId = null;
    }
}

function resetApp() {
    if (app) {
        app.reset();
        console.log('Emulator reset!');
    }
}