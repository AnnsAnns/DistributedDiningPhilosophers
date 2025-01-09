import { useState, useEffect } from "react";
import "./App.css";
import "tailwindcss/tailwind.css";

const DEV_MODE = false;

interface Philosopher {
  username: string;
  ip: string;
  port: number;
  of_type: string;
  state: string;
}

interface Cutlery {
  username: string;
  ip: string;
  port: number;
  of_type: string;
  state: Record<string, boolean>;
}

interface Response {
  phillosophers: Philosopher[];
  cutlery: Cutlery[];
  state_stats: Record<string, number>;
}

function App() {
  const [data, setData] = useState<Response | null>(null);
  const [prevData, setPrevData] = useState<Response | null>(null);
  const stateToColor = (state: string) => {
    switch (state) {
      case "PhilosopherHungry":
        return "bg-red-200";
      case "PhilosopherThinking":
        return "bg-blue-200";
      case "PhilosopherEating":
        return "bg-green-200";
      default:
        return "bg-gray-100";
    }
  };

  const stateToEmoji = (state: string) => {
    switch (state) {
      case "PhilosopherHungry":
        return "😠🍽️";
      case "PhilosopherThinking":
        return "🤔💬";
      case "PhilosopherEating":
        return "🥹🍲";
      default:
        return "❓";
    }
  };

  useEffect(() => {
    const fetchData = async () => {
      try {
        const response = await fetch("/api");
        const parsed_response: Response = await response.json();

        if (!hasChanged(data, parsed_response)) {
          return;
        }

        setPrevData(data);
        setData(parsed_response);
      } catch (_) {
        console.error("Failed to fetch data");
      }
    };

    const intervalId = setInterval(fetchData, 100);

    return () => clearInterval(intervalId);
  }, [data]);

  const hasChanged = (prev: unknown, current: unknown) => {
    return JSON.stringify(prev) !== JSON.stringify(current);
  };

  return (
    <div className="w-full m-8 ml-16 flex flex-col justify-center items-center">
      <h1 className="text-4xl font-bold mb-4">Dining Philosophers</h1>
      <h2 className="text-2xl font-bold mb-4">Stats:</h2>
      <div className="mb-4">
        {data && (
          <div>
            {Object.entries(data.state_stats).map(([key, value]) => {
              const hasChanged =
                prevData && prevData.state_stats[key] !== value;

              return (
                <div
                  key={key}
                  className={`mb-2 ${hasChanged ? "bg-yellow-200" : ""}`}
                >
                  <strong>{key}:</strong> {value}
                </div>
              );
            })}
          </div>
        )}
      </div>

      <h2 className="text-2xl flex items-center justify-center font-bold mb-4">
        Fetched Data:
      </h2>
      {data ? (
        <div>
          <div className="mb-4 flex justify-center">
            <ul className="list-none pl-5 h-full w-fit grid grid-cols-6 gap-3">
              {data.phillosophers.map((philosopher, index) => (
                <li
                  key={index}
                  className={`h-48 w-64 mb-2 flex flex-col justify-center text-center outline shadow-md rounded-md ${stateToColor(
                    philosopher.state
                  )} ${
                    prevData &&
                    hasChanged(prevData.phillosophers[index], philosopher)
                      ? "outline-yellow-300 outline-8"
                      : "outline-gray-800"
                  }`}
                >
                  <div className="text-lg font-sans font-semibold underline">
                    {philosopher.username}
                  </div>
                  <div>
                    <strong>IP:</strong> {philosopher.ip}
                  </div>
                  <div>
                    <strong>Port:</strong> {philosopher.port}
                  </div>
                  {/* <div><strong>Type:</strong> {philosopher.of_type}</div> */}
                  <div>
                    <strong>State:</strong> {philosopher.state}
                  </div>
                  <div className="text-2xl">
                    {stateToEmoji(philosopher.state)}
                  </div>
                </li>
              ))}
            </ul>
          </div>
          <div>
            <h3 className="text-xl font-semibold">Cutlery</h3>
            <ul className="list-none pl-5 grid grid-cols-6 gap-5">
              {data.cutlery.map((item, index) => (
                <li
                  key={index}
                  className={`mb-2  ${
                    prevData && hasChanged(prevData.cutlery[index], item)
                      ? "bg-yellow-200"
                      : ""
                  }`}
                >
                  <div>
                    <strong>Username:</strong> {item.username}
                  </div>
                  <div>
                    <strong>IP:</strong> {item.ip}
                  </div>
                  <div>
                    <strong>Port:</strong> {item.port}
                  </div>
                  <div>
                    <strong>Type:</strong> {item.of_type}
                  </div>
                  <div>
                    <strong>State:</strong> {JSON.stringify(item.state)}{" "}
                  </div>
                </li>
              ))}
            </ul>
          </div>
        </div>
      ) : (
        <div>Loading...</div>
      )}
    </div>
  );
}

export default App;
