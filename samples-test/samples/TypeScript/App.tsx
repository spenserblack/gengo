import React, { useState } from 'react';

function App() {
    const [count, setCount] = useState(0);
    return (
        <div className="App">
            <header className="App-header">
                <h1>Welcome to My React App</h1>
                <p>
                    Start editing to see some magic happen!
                </p>
                <div>
                    <p>Count: {count}</p>
                    <button onClick={() => setCount(count + 1)}>
                        Increment
                    </button>
                    <button onClick={() => setCount(count - 1)}>
                        Decrement
                    </button>
                </div>
            </header>
        </div>
    );
}

export default App;