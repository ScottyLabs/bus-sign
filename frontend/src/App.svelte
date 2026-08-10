<script lang="ts">
    import { onMount } from "svelte";
    import Header from "./lib/Header.svelte";
    import Footer from "./lib/Footer.svelte";
    import BusList from "./lib/BusList.svelte";

    type RouteInformation = {
        route: string;
        destination: string;
        arrivals: {
            bus_id: string;
            capacity: string;
            seconds: number;
        }[];
    };

    type PredictionsApiResponse = {
        [stopId: string]: RouteInformation[];
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
    const PREDICTIONS_REFRESH_MS = 3_000;
    const WEATHER_REFRESH_MS = 60_000;

    const INBOUND_STOP = "4407";
    const OUTBOUND_STOP = "7117";

    let inboundEntries: RouteInformation[] = [];
    let outboundEntries: RouteInformation[] = [];

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

    const sortByArrival = (entries: RouteInformation[]) =>
        [...entries].sort(
            (a, b) =>
                (a.arrivals[0]?.seconds ?? Infinity) -
                (b.arrivals[0]?.seconds ?? Infinity),
        );

    const formatClock = (now: Date) => {
        date = now.toLocaleDateString("en-US", {
            timeZone: "America/New_York",
            month: "2-digit",
            day: "2-digit",
            year: "2-digit",
        });
        time = now.toLocaleTimeString("en-US", {
            timeZone: "America/New_York",
            hour: "numeric",
            minute: "2-digit",
        });
    };

    const formatLastUpdated = (now: Date) =>
        now.toLocaleTimeString("en-US", {
            timeZone: "America/New_York",
            hour: "numeric",
            minute: "2-digit",
        });

    const fetchPredictions = async (): Promise<PredictionsApiResponse> => {
        const response = await fetch(`${API_BASE}/predictions`, {
            cache: "no-store",
        });

        if (!response.ok) {
            throw new Error(`Failed to fetch predictions: ${response.status}`);
        }

        return (await response.json()) as PredictionsApiResponse;
    };

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

    const refreshPredictions = async () => {
        try {
            const data = await fetchPredictions();
            inboundEntries = sortByArrival(data[INBOUND_STOP] ?? []);
            outboundEntries = sortByArrival(data[OUTBOUND_STOP] ?? []);
            lastUpdated = formatLastUpdated(new Date());
        } catch (error) {
            console.error(error);
        }
    };

    const refreshWeather = async () => {
        try {
            weather = await fetchWeather();
        } catch (error) {
            console.error(error);
        }
    };

    onMount(() => {
        formatClock(new Date());
        void refreshPredictions();
        void refreshWeather();

        const predictionsInterval = setInterval(
            refreshPredictions,
            PREDICTIONS_REFRESH_MS,
        );
        const weatherInterval = setInterval(refreshWeather, WEATHER_REFRESH_MS);
        const clockInterval = setInterval(() => formatClock(new Date()), 1_000);

        return () => {
            clearInterval(predictionsInterval);
            clearInterval(weatherInterval);
            clearInterval(clockInterval);
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
            stopId={INBOUND_STOP}
            walkMins={5}
            near="Tepper"
            entries={inboundEntries}
        />
        <div class="w-1 shrink-0 bg-light-gray"></div>
        <BusList
            title="Outbound"
            direction="outbound"
            stopId={OUTBOUND_STOP}
            walkMins={3}
            near="the UC"
            entries={outboundEntries}
        />
    </div>
    <Footer {lastUpdated} />
</main>
