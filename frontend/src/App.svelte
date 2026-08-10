<script lang="ts">
    import { onMount } from "svelte";
    import Header from "./lib/Header.svelte";
    import Footer from "./lib/Footer.svelte";
    import BusList from "./lib/BusList.svelte";

    type RouteInformation = {
        route: string;
        destination: string;
        scheduled?: boolean;
        arrivals: {
            bus_id: string;
            capacity: string;
            seconds: number;
        }[];
    };

    type WeatherApiResponse = {
        icon: string;
        condition: string;
        condition_until?: string;
        temp_f: number;
        high_f: number;
        low_f: number;
    };

    type Weather = {
        icon: string;
        condition: string;
        conditionUntil: string;
        tempF: number;
        highF: number;
        lowF: number;
    };

    const API_BASE = import.meta.env.VITE_API_BASE || "";
    const WEATHER_REFRESH_MS = 60_000;
    const MOCK_TICK_MS = 2_000;
    const MOCK_TICK_SECONDS = 45;

    let inboundEntries: RouteInformation[] = [
        {
            route: "61B",
            destination: "Braddock-Swissvale to Downtown",
            arrivals: [{ bus_id: "1201", capacity: "EMPTY", seconds: 20 }],
        },
        {
            route: "67",
            destination: "Forbes Hospital",
            scheduled: true,
            arrivals: [{ bus_id: "2401", capacity: "EMPTY", seconds: 420 }],
        },
        {
            route: "28X",
            destination: "Pittsburgh International Airport",
            arrivals: [
                { bus_id: "2801", capacity: "HALF_EMPTY", seconds: 720 },
            ],
        },
        {
            route: "61A",
            destination: "North Braddock to Downtown",
            arrivals: [{ bus_id: "1101", capacity: "EMPTY", seconds: 780 }],
        },
        {
            route: "61D",
            destination: "Murray Short to Oakland",
            arrivals: [{ bus_id: "1401", capacity: "FULL", seconds: 1020 }],
        },
        {
            route: "61C",
            destination: "McKeesport Homestead to Downtown",
            arrivals: [{ bus_id: "1301", capacity: "FULL", seconds: 1020 }],
        },
    ];

    let outboundEntries: RouteInformation[] = [
        {
            route: "61D",
            destination: "Murray Short to Waterfront",
            arrivals: [{ bus_id: "1402", capacity: "FULL", seconds: 120 }],
        },
        {
            route: "61A",
            destination: "North Braddock Via Braddock Hills Shopping Center",
            arrivals: [{ bus_id: "1102", capacity: "FULL", seconds: 900 }],
        },
        {
            route: "67",
            destination: "Forbes Ave Opposite Craig St",
            scheduled: true,
            arrivals: [
                { bus_id: "2402", capacity: "HALF_EMPTY", seconds: 420 },
            ],
        },
    ];

    const sortByArrival = (entries: RouteInformation[]) =>
        [...entries].sort(
            (a, b) =>
                (a.arrivals[0]?.seconds ?? Infinity) -
                (b.arrivals[0]?.seconds ?? Infinity),
        );

    const tickMockEntries = (entries: RouteInformation[]) => {
        const next = entries.map((entry) => ({
            ...entry,
            arrivals: entry.arrivals.map((arrival) => {
                let seconds = arrival.seconds - MOCK_TICK_SECONDS;
                if (seconds <= 0) {
                    seconds = 480 + Math.floor(Math.random() * 720);
                }
                return { ...arrival, seconds };
            }),
        }));

        // Occasionally pull a later bus forward so rows swap mid-list
        if (Math.random() < 0.45 && next.length > 1) {
            const i = 1 + Math.floor(Math.random() * (next.length - 1));
            const arrival = next[i].arrivals[0];
            if (arrival) {
                arrival.seconds = Math.max(
                    15,
                    arrival.seconds - 150 - Math.floor(Math.random() * 240),
                );
            }
        }

        return sortByArrival(next);
    };

    let weather: Weather = {
        icon: "01d",
        condition: "-",
        conditionUntil: "",
        tempF: 0,
        highF: 0,
        lowF: 0,
    };
    let date = "";
    let time = "";
    let lastUpdated = "";

    const formatClock = (now: Date) => {
        date = now.toLocaleDateString("en-US", {
            month: "2-digit",
            day: "2-digit",
            year: "2-digit",
        });
        time = now.toLocaleTimeString("en-US", {
            hour: "numeric",
            minute: "2-digit",
        });
    };

    const formatLastUpdated = (now: Date) =>
        now.toLocaleTimeString("en-US", {
            hour: "numeric",
            minute: "2-digit",
        });

    const fetchWeather = async (): Promise<Weather> => {
        const response = await fetch(`${API_BASE}/weather`, {
            cache: "no-store",
        });

        if (!response.ok) {
            throw new Error(`Failed to fetch weather: ${response.status}`);
        }

        const data = (await response.json()) as WeatherApiResponse;
        return {
            icon: data.icon,
            condition: data.condition,
            conditionUntil: data.condition_until ?? "",
            tempF: data.temp_f,
            highF: data.high_f,
            lowF: data.low_f,
        };
    };

    const refreshWeather = async () => {
        try {
            weather = await fetchWeather();
        } catch (error) {
            console.error(error);
        }
    };

    onMount(() => {
        const now = new Date();
        formatClock(now);
        lastUpdated = formatLastUpdated(now);
        void refreshWeather();
        inboundEntries = sortByArrival(inboundEntries);
        outboundEntries = sortByArrival(outboundEntries);

        const weatherInterval = setInterval(refreshWeather, WEATHER_REFRESH_MS);
        const clockInterval = setInterval(() => formatClock(new Date()), 1_000);
        const mockInterval = setInterval(() => {
            inboundEntries = tickMockEntries(inboundEntries);
            outboundEntries = tickMockEntries(outboundEntries);
            lastUpdated = formatLastUpdated(new Date());
        }, MOCK_TICK_MS);

        return () => {
            clearInterval(weatherInterval);
            clearInterval(clockInterval);
            clearInterval(mockInterval);
        };
    });
</script>

<main>
    <Header
        icon={weather.icon}
        condition={weather.condition}
        conditionUntil={weather.conditionUntil}
        tempF={weather.tempF}
        highF={weather.highF}
        lowF={weather.lowF}
        {date}
        {time}
    />
    <div
        class="flex flex-1 min-h-0 flex-row items-stretch px-13.5 pt-12 pb-10 gap-11 w-full"
    >
        <BusList
            title="Inbound"
            direction="inbound"
            stopId="4407"
            walkMins={5}
            near="Tepper"
            nextMajorRoutes={[
                { route: "28X", minutes: 50 },
                { route: "58", minutes: 65 },
            ]}
            entries={inboundEntries}
        />
        <div class="w-1 shrink-0 bg-light-gray"></div>
        <BusList
            title="Outbound"
            direction="outbound"
            stopId="7117"
            walkMins={3}
            near="the UC"
            nextMajorRoutes={[
                { route: "28X", minutes: 40 },
                { route: "69", minutes: 35 },
            ]}
            entries={outboundEntries}
        />
    </div>
    <Footer {lastUpdated} />
</main>
