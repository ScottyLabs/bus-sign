<script lang="ts">
    import { onMount } from "svelte";
    import QRCode from "qrcode";
    import BusTimeEntry from "./lib/BusTimeEntry.svelte";

    type RouteInformation = {
        route: string;
        destination: string;
        arrivals: {
            bus_id: string;
            capacity: string;
            seconds: number;
        }[];
    };

    type APIResponse = {
        [stopId: string]: RouteInformation[];
    };

    type WeatherData = {
        temperature_f: number;
        weather_code: number;
        is_day: boolean;
        summary: string;
        detail: string;
    };

    type ScheduleData = {
        [stopId: string]: NextRouteSummary[];
    };

    type NextRouteSummary = {
        route: string;
        destination: string;
        seconds: number;
        scheduled?: boolean;
    };

    type DisplayRow = {
        key: string;
        entry: RouteInformation | null;
    };

    type CrossingBus = {
        id: number;
        color: string;
        direction: "left-to-right" | "right-to-left";
    };

    let entriesUC: RouteInformation[] = [];
    let entriesTep: RouteInformation[] = [];
    let lastUpdated: Date | null = null;
    let directionIndex = 0;
    let nextRouteIndex = 0;
    let qrCodeUC = "";
    let qrCodeTep = "";
    let qrCode28X = "";
    let crossingBuses: CrossingBus[] = [];
    let nextCrossingBusId = 0;
    let weather: WeatherData | null = null;
    let twentyEightXSchedule: ScheduleData = {};
    let weatherError = false;
    let weatherClockTick = Date.now();

    const UC_SUBLABELS = ["UC side of Forbes", "Toward Squirrel Hill", "Heading East", "Toward Beeler St"];
    const TEPPER_SUBLABELS = [
        "Tepper side of Forbes",
        "Toward Oakland (Univ. of Pitt)",
        "Heading West",
        "To Craig St",
    ];
    const MAJOR_ROUTES = ["61A", "61B", "61C", "61D", "67", "28X"];
    const MAIN_LIST_SECONDS_LIMIT = 30 * 60;
    const NEXT_MAJOR_SECONDS_LIMIT = 60 * 60;
    const MAIN_LIST_ENTRY_LIMIT = 6;
    const NEXT_MAJOR_VISIBLE_LIMIT = 5;
    const API_BASE = import.meta.env.VITE_API_BASE || "";
    const CROSSING_BUS_COLORS = ["#C41230", "#FDB515", "#009647", "#043673", "#008F91"];
    const ROUTE_COLORS: Record<string, string> = {
        "61A": "#C41230",
        "61B": "#FDB515",
        "61C": "#009647",
        "61D": "#043673",
        "67": "#008F91",
        "28X": "#050505",
    };
    const CROSSING_BUS_DURATION_MS = 20 * 1000;
    const WEATHER_UNIT_CYCLE_MS = 2 * 60 * 1000;
    const WEATHER_UNIT_F_DURATION_MS = 105 * 1000;
    const STARTUP_BUS_COUNT = 5;
    const STARTUP_BUS_WINDOW_MS = 30 * 1000;
    const UC_STOP_URL =
        "https://realtime.portauthority.org/bustime/eta/eta.jsp?route=---&direction=---&stop=---&id=7117&showAllBusses=on&findstop=on";
    const TEP_STOP_URL =
        "https://realtime.portauthority.org/bustime/eta/eta.jsp?route=---&direction=---&stop=---&id=4407&showAllBusses=on&findstop=on";
    const TWENTY_EIGHT_X_URL = "https://www.rideprt.org/pdfs/28X.pdf";

    const normalizeRoute = (route: string) => route.trim().toUpperCase();

    const isMajorRoute = (route: string) => MAJOR_ROUTES.includes(normalizeRoute(route));

    const getRouteColor = (route: string) => ROUTE_COLORS[normalizeRoute(route)] || null;

    const formatDestination = (route: string, destination: string) =>
        normalizeRoute(route) === "28X" && destination.toLowerCase().includes("airport")
            ? "(✈ PIT) Pittsburgh International Airport"
            : destination;

    const isWithinMainListWindow = (entry: RouteInformation) =>
        (entry.arrivals[0]?.seconds ?? Infinity) <= MAIN_LIST_SECONDS_LIMIT;

    const isWithinNextMajorWindow = (entry: RouteInformation) => {
        const seconds = entry.arrivals[0]?.seconds ?? Infinity;
        return seconds <= MAIN_LIST_SECONDS_LIMIT || (isMajorRoute(entry.route) && seconds <= NEXT_MAJOR_SECONDS_LIMIT);
    };

    const getMainEntries = (entries: RouteInformation[]) =>
        entries.filter(isWithinMainListWindow).slice(0, MAIN_LIST_ENTRY_LIMIT);

    const getDisplayRows = (entries: RouteInformation[]): DisplayRow[] => [
        ...entries.map((entry, index) => ({
            key: `${entry.route}-${entry.destination}-${index}`,
            entry,
        })),
        ...Array.from({ length: Math.max(MAIN_LIST_ENTRY_LIMIT - entries.length, 0) }, (_, index) => ({
            key: `placeholder-${index}`,
            entry: null,
        })),
    ];

    const formatTimeRemaining = (seconds: number): string => {
        if (seconds < 60) return "NOW";
        return `${Math.ceil(seconds / 60)}m`;
    };

    const getHiddenMajorRouteSummaries = (
        stopId: string,
        entries: RouteInformation[],
        mainEntries: RouteInformation[],
    ): NextRouteSummary[] => {
        const visibleEntries = new Set(mainEntries);
        const hasVisible28X = mainEntries.some((entry) => normalizeRoute(entry.route) === "28X");
        const scheduledCandidates = twentyEightXSchedule[stopId] || [];
        const nextScheduled28X = scheduledCandidates[hasVisible28X ? 1 : 0];

        const hiddenRoutes = entries
            .filter(
                (entry) =>
                    !visibleEntries.has(entry) &&
                    entry.arrivals[0] &&
                    isWithinNextMajorWindow(entry) &&
                    normalizeRoute(entry.route) !== "28X",
            )
            .sort(
                (a, b) =>
                    (a.arrivals[0]?.seconds ?? Infinity) -
                    (b.arrivals[0]?.seconds ?? Infinity),
            )
            .map((entry) => {
                return {
                    route: normalizeRoute(entry.route),
                    destination: formatDestination(entry.route, entry.destination),
                    seconds: entry.arrivals[0]?.seconds ?? Infinity,
                };
            });

        if (!nextScheduled28X) return hiddenRoutes;

        return [
            nextScheduled28X,
            ...hiddenRoutes,
        ];
    };

    const getVisibleNextRoutes = (routes: NextRouteSummary[]) => {
        if (routes.length <= NEXT_MAJOR_VISIBLE_LIMIT) return routes;
        const scheduled28X = routes.find((route) => route.route === "28X" && route.scheduled);
        const rotatingRoutes = routes.filter((route) => route !== scheduled28X);

        if (!scheduled28X) {
            const start = (nextRouteIndex * NEXT_MAJOR_VISIBLE_LIMIT) % routes.length;
            return Array.from(
                { length: NEXT_MAJOR_VISIBLE_LIMIT },
                (_, offset) => routes[(start + offset) % routes.length],
            );
        }

        const rotatingVisibleLimit = Math.max(NEXT_MAJOR_VISIBLE_LIMIT - 1, 0);
        if (rotatingVisibleLimit === 0) return [scheduled28X];

        const start =
            rotatingRoutes.length > 0
                ? (nextRouteIndex * rotatingVisibleLimit) % rotatingRoutes.length
                : 0;

        return [
            scheduled28X,
            ...Array.from(
                { length: Math.min(rotatingVisibleLimit, rotatingRoutes.length) },
                (_, offset) => rotatingRoutes[(start + offset) % rotatingRoutes.length],
            ),
        ];
    };

    const fetchPredictions = async (): Promise<APIResponse> => {
        const response = await fetch(`${API_BASE}/predictions`, {
            cache: "no-store",
        });

        if (!response.ok) {
            throw new Error(`Failed to fetch predictions: ${response.status}`);
        }

        const data = (await response.json()) as APIResponse;
        return data;
    };

    const fetchWeather = async (): Promise<WeatherData> => {
        const response = await fetch(`${API_BASE}/weather`, {
            cache: "no-store",
        });

        if (!response.ok) {
            throw new Error(`Failed to fetch weather: ${response.status}`);
        }

        return (await response.json()) as WeatherData;
    };

    const fetchTwentyEightXSchedule = async (): Promise<ScheduleData> => {
        const response = await fetch(`${API_BASE}/schedule/28x`, {
            cache: "no-store",
        });

        if (!response.ok) {
            throw new Error(`Failed to fetch 28X schedule: ${response.status}`);
        }

        return (await response.json()) as ScheduleData;
    };

    const refresh = async () => {
        try {
            const data = await fetchPredictions();
            entriesUC = (data["7117"] || []).sort(
                (a, b) =>
                    (a.arrivals[0]?.seconds || Infinity) -
                    (b.arrivals[0]?.seconds || Infinity),
            );
            entriesTep = (data["4407"] || []).sort(
                (a, b) =>
                    (a.arrivals[0]?.seconds || Infinity) -
                    (b.arrivals[0]?.seconds || Infinity),
            );
            lastUpdated = new Date();
        } catch (error) {
            console.error(error);
        }
    };

    const refreshWeather = async () => {
        try {
            weather = await fetchWeather();
            weatherError = false;
        } catch (error) {
            weatherError = true;
            console.error(error);
        }
    };

    const refreshTwentyEightXSchedule = async () => {
        try {
            twentyEightXSchedule = await fetchTwentyEightXSchedule();
        } catch (error) {
            console.error(error);
        }
    };

    const generateQrCodes = async () => {
        try {
            const options = {
                errorCorrectionLevel: "L" as const,
                margin: 1,
                width: 112,
                color: {
                    dark: "#050505",
                    light: "#ffffff",
                },
            };

            [qrCodeUC, qrCodeTep, qrCode28X] = await Promise.all([
                QRCode.toDataURL(UC_STOP_URL, options),
                QRCode.toDataURL(TEP_STOP_URL, options),
                QRCode.toDataURL(TWENTY_EIGHT_X_URL, options),
            ]);
        } catch (error) {
            console.error(error);
        }
    };

    const getRandomCrossingBusColor = () =>
        CROSSING_BUS_COLORS[Math.floor(Math.random() * CROSSING_BUS_COLORS.length)];

    const shuffleItems = <T,>(items: T[]): T[] =>
        [...items].sort(() => Math.random() - 0.5);

    const getStartupBusColors = () => [
        ...shuffleItems(CROSSING_BUS_COLORS),
        getRandomCrossingBusColor(),
    ];

    const getStartupBusDirections = (): CrossingBus["direction"][] => {
        const majorityDirection = Math.random() > 0.5 ? "left-to-right" : "right-to-left";
        const minorityDirection =
            majorityDirection === "left-to-right" ? "right-to-left" : "left-to-right";

        return shuffleItems([
            majorityDirection,
            majorityDirection,
            majorityDirection,
            minorityDirection,
            minorityDirection,
        ]);
    };

    const getWeatherIconClass = (weatherCode: number, isDay: boolean): string => {
        switch (weatherCode) {
            case 0:
                return isDay ? "wi-day-sunny" : "wi-night-clear";
            case 1:
            case 2:
                return isDay ? "wi-day-cloudy" : "wi-night-alt-cloudy";
            case 3:
                return "wi-cloudy";
            case 45:
            case 48:
                return isDay ? "wi-day-fog" : "wi-night-fog";
            case 51:
            case 53:
            case 55:
            case 56:
            case 57:
                return isDay ? "wi-day-sprinkle" : "wi-night-alt-sprinkle";
            case 61:
            case 63:
            case 65:
            case 66:
            case 67:
                return isDay ? "wi-day-rain" : "wi-night-alt-rain";
            case 71:
            case 73:
            case 75:
            case 77:
                return isDay ? "wi-day-snow" : "wi-night-alt-snow";
            case 80:
            case 81:
            case 82:
                return isDay ? "wi-day-showers" : "wi-night-alt-showers";
            case 85:
            case 86:
                return isDay ? "wi-day-snow" : "wi-night-alt-snow";
            case 95:
                return isDay ? "wi-day-thunderstorm" : "wi-night-alt-thunderstorm";
            case 96:
            case 99:
                return isDay ? "wi-day-hail" : "wi-night-alt-hail";
            default:
                return isDay ? "wi-day-cloudy" : "wi-night-alt-cloudy";
        }
    };

    const startCrossingBus = (
        color = getRandomCrossingBusColor(),
        direction: CrossingBus["direction"] = Math.random() > 0.5 ? "left-to-right" : "right-to-left",
    ) => {
        const bus = {
            id: nextCrossingBusId,
            color,
            direction,
        } satisfies CrossingBus;

        nextCrossingBusId += 1;
        crossingBuses = [...crossingBuses, bus];

        return window.setTimeout(() => {
            crossingBuses = crossingBuses.filter((crossingBus) => crossingBus.id !== bus.id);
        }, CROSSING_BUS_DURATION_MS);
    };

    const shouldStartScheduledBus = (date: Date) => {
        const minute = date.getMinutes();
        const second = date.getSeconds();
        const isHalfHourMark = minute === 0 || minute === 30;
        const isQuarterHourMark = minute === 15 || minute === 45;

        return (isHalfHourMark && (second === 0 || second === 30)) || (isQuarterHourMark && second === 0);
    };

    $: formattedTime = lastUpdated
        ? lastUpdated.toLocaleTimeString("en-US", {
              hour: "numeric",
              minute: "2-digit",
              second: "2-digit",
          })
        : "";
    $: displayDate = (lastUpdated || new Date()).toLocaleDateString("en-US", {
        month: "numeric",
        day: "numeric",
        year: "2-digit",
    });
    $: displayClock = (lastUpdated || new Date()).toLocaleTimeString("en-US", {
        hour: "numeric",
        minute: "2-digit",
    });
    $: mainEntriesUC = getMainEntries(entriesUC);
    $: mainEntriesTep = getMainEntries(entriesTep);
    $: displayRowsUC = getDisplayRows(mainEntriesUC);
    $: displayRowsTep = getDisplayRows(mainEntriesTep);
    $: nextRoutesUC = getHiddenMajorRouteSummaries("7117", entriesUC, mainEntriesUC);
    $: nextRoutesTep = getHiddenMajorRouteSummaries("4407", entriesTep, mainEntriesTep);
    $: visibleNextRoutesUC = getVisibleNextRoutes(nextRoutesUC);
    $: visibleNextRoutesTep = getVisibleNextRoutes(nextRoutesTep);
    $: ucSublabel = UC_SUBLABELS[directionIndex % UC_SUBLABELS.length];
    $: tepperSublabel = TEPPER_SUBLABELS[directionIndex % TEPPER_SUBLABELS.length];
    $: weatherIconClass = weather ? getWeatherIconClass(weather.weather_code, weather.is_day) : "wi-day-cloudy";
    $: weatherUnit = weatherClockTick % WEATHER_UNIT_CYCLE_MS < WEATHER_UNIT_F_DURATION_MS ? "F" : "C";
    $: weatherTemperatureValue =
        weather
            ? weatherUnit === "F"
                ? weather.temperature_f
                : Math.round(((weather.temperature_f - 32) * 5) / 9)
            : null;
    $: weatherTemperatureDisplay =
        weatherTemperatureValue !== null
            ? `${weatherTemperatureValue} ${String.fromCharCode(176)}${weatherUnit}`
            : `-- ${String.fromCharCode(176)}${weatherUnit}`;
    $: weatherSummaryText = weather?.summary || (weatherError ? "Weather offline" : "Loading weather");
    $: weatherDetailText = weather?.detail || (weatherError ? "Restart backend if needed" : "Fetching latest conditions");

    onMount(() => {
        void refresh();
        void refreshWeather();
        void refreshTwentyEightXSchedule();
        void generateQrCodes();
        const refreshInterval = setInterval(refresh, 3_000);
        const weatherInterval = setInterval(refreshWeather, 15 * 60 * 1000);
        const twentyEightXScheduleInterval = setInterval(refreshTwentyEightXSchedule, 7 * 24 * 60 * 60 * 1000);
        const weatherClockInterval = setInterval(() => {
            weatherClockTick = Date.now();
        }, 1_000);
        const directionInterval = setInterval(() => {
            directionIndex += 1;
        }, 5_000);
        const nextRouteInterval = setInterval(() => {
            nextRouteIndex += 1;
        }, 7_000);
        const crossingBusTimeouts: number[] = [];
        let lastScheduledBusKey = "";
        const startupBusColors = getStartupBusColors();
        const startupBusDirections = getStartupBusDirections();
        const startupBusTimeouts = Array.from(
            { length: STARTUP_BUS_COUNT },
            (_, index) =>
                window.setTimeout(() => {
                    crossingBusTimeouts.push(
                        startCrossingBus(startupBusColors[index], startupBusDirections[index]),
                    );
                }, index * (STARTUP_BUS_WINDOW_MS / STARTUP_BUS_COUNT)),
        );
        const scheduledBusInterval = setInterval(() => {
            const now = new Date();
            const scheduleKey = `${now.getHours()}:${now.getMinutes()}:${now.getSeconds()}`;

            if (scheduleKey !== lastScheduledBusKey && shouldStartScheduledBus(now)) {
                lastScheduledBusKey = scheduleKey;
                crossingBusTimeouts.push(startCrossingBus());
            }
        }, 500);

        return () => {
            clearInterval(refreshInterval);
            clearInterval(weatherInterval);
            clearInterval(twentyEightXScheduleInterval);
            clearInterval(weatherClockInterval);
            clearInterval(directionInterval);
            clearInterval(nextRouteInterval);
            clearInterval(scheduledBusInterval);
            for (const timeout of startupBusTimeouts) window.clearTimeout(timeout);
            for (const timeout of crossingBusTimeouts) window.clearTimeout(timeout);
        };
    });
</script>

<main>
    <section class="sign-shell" aria-label="Live PRT bus arrivals">
        <header class="top-bar">
            <div class="brand-lockup">
                <img
                    src="/cmu-wordmark-horizontal-r.png"
                    alt="Carnegie Mellon University"
                    class="header-wordmark"
                />
            </div>
            <div class="header-location">Live PRT Bus Transit Times</div>
            <div class="header-status">
                <div
                    class="weather-chip"
                    aria-label={`${weatherSummaryText}, ${weatherTemperatureDisplay}, ${weatherDetailText}`}
                >
                    <i class={`weather-icon wi ${weatherIconClass}`} aria-hidden="true"></i>
                    <div class="weather-copy">
                        <div class="weather-topline">
                            <strong>{weatherSummaryText}</strong>
                            <span class="weather-divider"></span>
                            <span class="weather-temp">{weatherTemperatureDisplay}</span>
                        </div>
                        <div class="weather-detail">{weatherDetailText}</div>
                    </div>
                </div>
                <div class="date-mark">{displayDate}<br />{displayClock}</div>
            </div>
        </header>

        <div class="display">
            <aside class="pattern-rail" aria-hidden="true"></aside>

            <div class="board-content">
                <div class="board-grid">
                <section class="stop-panel" aria-labelledby="uc-side-heading">
                    <h1 id="uc-side-heading">
                        <span class="heading-title">
                            <span class="heading-main">
                                <span>From Downtown</span>
                                <small class="stop-number">Stop #7117</small>
                            </span>
                        </span>
                        <span class="heading-meta">
                            <small>{ucSublabel}</small>
                        </span>
                    </h1>

                    <div class="arrival-list">
                        {#each displayRowsUC as row (row.key)}
                            {#if row.entry}
                                <BusTimeEntry
                                    {...row.entry}
                                    routeColor={getRouteColor(row.entry.route)}
                                    highlightStartSeconds={60}
                                    highlightEndSeconds={120}
                                />
                            {:else}
                                <BusTimeEntry route="" destination="" arrivals={[]} placeholder={true} />
                            {/if}
                        {/each}
                    </div>

                    <div class="next-summary" aria-label="Next hidden major routes for UC Side">
                        <p>Next major routes</p>
                        <div class="next-route-grid">
                            {#if visibleNextRoutesUC.length > 0}
                                {#each visibleNextRoutesUC as entry (entry.route + entry.destination + entry.seconds)}
                                    <div class="next-route" class:scheduled-route={entry.scheduled}>
                                        <strong style:color={getRouteColor(entry.route)}>
                                            {entry.route}
                                            {#if entry.scheduled}
                                                <em>Scheduled</em>
                                            {/if}
                                        </strong>
                                        <span>{formatTimeRemaining(entry.seconds)}</span>
                                        <small>{entry.destination}</small>
                                    </div>
                                {/each}
                            {:else}
                                <div class="next-route next-empty-route">
                                    <strong>###</strong>
                                    <span>---</span>
                                    <small>No additional major routes</small>
                                </div>
                            {/if}
                        </div>
                    </div>
                    {#if qrCodeUC}
                        <div class="stop-qr-cluster">
                            {#if qrCode28X}
                                <a class="stop-qr stop-qr-secondary" href={TWENTY_EIGHT_X_URL} target="_blank" rel="noreferrer">
                                    <span>Find your next 28X here!</span>
                                    <img src={qrCode28X} alt="QR code for the 28X Airport Flyer timetable PDF" />
                                </a>
                            {/if}
                            <a class="stop-qr" href={UC_STOP_URL} target="_blank" rel="noreferrer">
                                <span>Live stop info</span>
                                <img src={qrCodeUC} alt="QR code for live UC Side arrivals at stop 7117" />
                            </a>
                        </div>
                    {/if}
                </section>

                <section class="stop-panel" aria-labelledby="tepper-side-heading">
                    <h2 id="tepper-side-heading">
                        <span class="heading-title">
                            <span class="heading-main">
                                <span>To Downtown</span>
                                <small class="stop-number">Stop #4407</small>
                            </span>
                        </span>
                        <span class="heading-meta">
                            <small>{tepperSublabel}</small>
                        </span>
                    </h2>

                    <div class="arrival-list">
                        {#each displayRowsTep as row (row.key)}
                            {#if row.entry}
                                <BusTimeEntry
                                    {...row.entry}
                                    routeColor={getRouteColor(row.entry.route)}
                                    highlightStartSeconds={60}
                                    highlightEndSeconds={300}
                                />
                            {:else}
                                <BusTimeEntry route="" destination="" arrivals={[]} placeholder={true} />
                            {/if}
                        {/each}
                    </div>

                    <div class="next-summary" aria-label="Next hidden major routes for Tepper Side">
                        <p>Next major routes</p>
                        <div class="next-route-grid">
                            {#if visibleNextRoutesTep.length > 0}
                                {#each visibleNextRoutesTep as entry (entry.route + entry.destination + entry.seconds)}
                                    <div class="next-route" class:scheduled-route={entry.scheduled}>
                                        <strong style:color={getRouteColor(entry.route)}>
                                            {entry.route}
                                            {#if entry.scheduled}
                                                <em>Scheduled</em>
                                            {/if}
                                        </strong>
                                        <span>{formatTimeRemaining(entry.seconds)}</span>
                                        <small>{entry.destination}</small>
                                    </div>
                                {/each}
                            {:else}
                                <div class="next-route next-empty-route">
                                    <strong>###</strong>
                                    <span>---</span>
                                    <small>No additional major routes</small>
                                </div>
                            {/if}
                        </div>
                    </div>
                    {#if qrCodeTep}
                        <div class="stop-qr-cluster">
                            {#if qrCode28X}
                                <a class="stop-qr stop-qr-secondary" href={TWENTY_EIGHT_X_URL} target="_blank" rel="noreferrer">
                                    <span>Find your next 28X here!</span>
                                    <img src={qrCode28X} alt="QR code for the 28X Airport Flyer timetable PDF" />
                                </a>
                            {/if}
                            <a class="stop-qr" href={TEP_STOP_URL} target="_blank" rel="noreferrer">
                                <span>Live stop info</span>
                                <img src={qrCodeTep} alt="QR code for live Tepper Side arrivals at stop 4407" />
                            </a>
                        </div>
                    {/if}
                </section>
                </div>
            </div>
        </div>

        {#each crossingBuses as bus (bus.id)}
            <div
                class:travel-right={bus.direction === "left-to-right"}
                class:travel-left={bus.direction === "right-to-left"}
                class="crossing-bus"
                style:--bus-color={bus.color}
                aria-hidden="true"
            >
                <div class="crossing-bus-body">
                    <span class="crossing-bus-window"></span>
                    <span class="crossing-bus-window"></span>
                    <span class="crossing-bus-window"></span>
                </div>
                <span class="crossing-bus-wheel left"></span>
                <span class="crossing-bus-wheel right"></span>
            </div>
        {/each}

    </section>

    <footer class="footer">
        <img src="/scotty.svg" alt="Scotty Logo" class="footer-logo" />
        <p>
            Project by Undergraduate Student Senate via collaboration with ScottyLabs.
            Data provided under license from PRT.
        </p>
        {#if formattedTime}
            <p class="last-updated">Last updated: {formattedTime}</p>
        {/if}
    </footer>
</main>

<style>
    .sign-shell {
        position: relative;
        flex: 1 1 auto;
        display: flex;
        flex-direction: column;
        min-height: 0;
        overflow: hidden;
        background: #ffffff;
        color: #050505;
    }

    .top-bar {
        min-height: 60px;
        background: #bd1238;
        color: #fff;
        display: grid;
        grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
        align-items: center;
        gap: 20px;
        padding: 6px 28px;
        box-sizing: border-box;
    }

    .brand-lockup {
        display: flex;
        align-items: center;
        min-width: 0;
    }

    .header-wordmark {
        display: block;
        width: auto;
        height: 34px;
        max-width: min(100%, 340px);
        object-fit: contain;
        filter: brightness(0) invert(1);
        transform: translateY(3px);
    }

    .header-location {
        min-width: 0;
        font-size: clamp(22px, 1.9vw, 34px);
        font-weight: 600;
        line-height: 1;
        text-align: center;
        white-space: nowrap;
    }

    .header-status {
        display: flex;
        align-items: center;
        justify-content: flex-end;
        gap: 12px;
        min-width: 0;
    }

    .weather-chip {
        display: flex;
        align-items: center;
        gap: 10px;
        min-width: 0;
        padding: 6px 12px 7px;
        border-radius: 10px;
        background: rgba(143, 16, 42, 0.82);
        color: #ffffff;
    }

    .weather-icon {
        flex: 0 0 auto;
        font-size: 28px;
        line-height: 1;
        color: #ffffff;
    }

    .weather-copy {
        min-width: 0;
        display: flex;
        flex-direction: column;
        gap: 2px;
    }

    .weather-topline {
        display: flex;
        align-items: center;
        gap: 8px;
        min-width: 0;
        white-space: nowrap;
    }

    .weather-topline strong {
        font-size: clamp(14px, 1.15vw, 20px);
        font-weight: 600;
        line-height: 1;
    }

    .weather-divider {
        width: 2px;
        height: 22px;
        background: rgba(255, 255, 255, 0.35);
        border-radius: 999px;
    }

    .weather-temp {
        font-size: clamp(14px, 1.15vw, 20px);
        font-weight: 600;
        line-height: 1;
    }

    .weather-detail {
        font-size: clamp(10px, 0.8vw, 14px);
        line-height: 1;
        color: rgba(255, 255, 255, 0.9);
        white-space: nowrap;
    }

    .date-mark {
        font-size: 22px;
        line-height: 1.05;
        text-align: right;
        white-space: nowrap;
    }

    .footer-logo {
        width: auto;
        height: 24px;
        flex-shrink: 0;
    }

    .display {
        flex: 1 1 auto;
        min-height: 0;
        display: grid;
        grid-template-columns: 58px minmax(0, 1fr);
        background: #ffffff;
    }

    .pattern-rail {
        background: #111 url("/cmu-tartan-wave-full-color-crop-03.png") center center / cover no-repeat;
        border-right: 3px solid #222;
    }

    .board-content {
        min-height: 0;
        display: flex;
        flex-direction: column;
        padding: 24px 44px 54px;
        box-sizing: border-box;
    }

    .board-grid {
        min-height: 0;
        flex: 1 1 auto;
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 42px;
    }

    .stop-panel {
        position: relative;
        min-width: 0;
        display: flex;
        flex-direction: column;
        padding-bottom: 22000px;
    }

    .stop-panel + .stop-panel {
        border-left: 2px solid rgba(5, 5, 5, 0.32);
        padding-left: 42px;
    }

    h1,
    h2 {
        margin: 0 0 10px;
        font-size: clamp(28px, 3.1vw, 52px);
        line-height: 1;
        color: #050505;
        letter-spacing: 0;
        display: flex;
        align-items: baseline;
        justify-content: space-between;
        gap: 18px;
    }

    .heading-title,
    .heading-meta {
        min-width: 0;
        display: flex;
        align-items: baseline;
        gap: 12px;
    }

    .heading-title {
        flex: 1 1 auto;
        align-items: baseline;
    }

    .heading-main {
        min-width: 0;
        display: flex;
        align-items: baseline;
        gap: 14px;
    }

    .heading-main > span {
        flex: 0 0 auto;
        font-size: clamp(22px, 2.1vw, 40px);
        line-height: 0.95;
    }

    .heading-meta {
        flex: 0 0 auto;
        align-items: flex-end;
    }

    h1 small,
    h2 small {
        color: #404040;
        font-size: clamp(13px, 1.1vw, 18px);
        line-height: 1;
        white-space: nowrap;
    }

    .stop-number {
        color: #707070;
        font-size: clamp(13px, 1.1vw, 18px);
        font-style: italic;
        line-height: 1;
        text-align: left;
        white-space: nowrap;
    }

    .arrival-list {
        display: flex;
        flex-direction: column;
        gap: 0;
        min-height: 0;
    }

    .next-summary {
        margin-top: auto;
        padding-top: 22px;
        color: #1a1a1a;
    }

    .next-summary p {
        margin: 0 0 8px;
        color: #575757;
        font-size: 13px;
        line-height: 1;
        text-transform: uppercase;
    }

    .next-route-grid {
        display: grid;
        grid-template-columns: repeat(3, minmax(0, 1fr));
        padding-right: 112px;
        gap: 8px;
    }

    .next-route {
        min-width: 0;
        border-top: 1px solid rgba(5, 5, 5, 0.26);
        padding-top: 8px;
        display: grid;
        grid-template-columns: auto minmax(0, 1fr);
        gap: 2px 8px;
        align-items: baseline;
        color: #101010;
    }

    .scheduled-route {
        background: #fff3bf;
        border-top-color: #d5b13d;
        border-radius: 8px;
        padding: 8px 10px 6px;
    }

    .next-route strong {
        color: #ffc627;
        display: flex;
        align-items: baseline;
        gap: 6px;
        font-size: 18px;
        line-height: 1;
    }

    .next-route strong em {
        color: #8a6a00;
        font-size: 10px;
        font-style: normal;
        letter-spacing: 0.04em;
        text-transform: uppercase;
    }

    .next-route span {
        color: #050505;
        font-size: 15px;
        line-height: 1;
        white-space: nowrap;
    }

    .next-route small {
        grid-column: 1 / -1;
        min-width: 0;
        color: #005f99;
        font-size: 11px;
        line-height: 1.1;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .next-empty-route {
        opacity: 0.7;
    }

    .next-empty-route strong,
    .next-empty-route span,
    .next-empty-route small {
        color: #888;
    }

    .stop-qr-cluster {
        position: absolute;
        right: 0;
        bottom: 192px;
        z-index: 3;
        display: flex;
        align-items: flex-end;
        gap: 10px;
    }

    .stop-qr {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 5px;
        width: 88px;
        color: #050505;
        font-size: 10px;
        line-height: 1.05;
        text-align: center;
        text-decoration: none;
        white-space: nowrap;
    }

    .stop-qr-secondary {
        width: 102px;
        font-size: 9px;
        line-height: 1.1;
    }

    .stop-qr span {
        width: 88px;
    }

    .stop-qr-secondary span {
        width: 102px;
    }

    .stop-qr img {
        display: block;
        width: 88px;
        height: 88px;
        image-rendering: pixelated;
    }

    .crossing-bus {
        --bus-color: #bd1238;
        position: absolute;
        bottom: 0;
        left: 0;
        z-index: 1;
        width: 96px;
        height: 50px;
        pointer-events: none;
        animation-duration: 20s;
        animation-timing-function: linear;
        animation-fill-mode: forwards;
    }

    .crossing-bus.travel-right {
        animation-name: bus-cross-right;
    }

    .crossing-bus.travel-left {
        animation-name: bus-cross-left;
    }

    .crossing-bus-body {
        position: absolute;
        left: 4px;
        bottom: 7px;
        width: 88px;
        height: 34px;
        border-radius: 8px 8px 5px 5px;
        background: var(--bus-color);
        border: 3px solid #ffffff;
        box-sizing: border-box;
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 6px;
        padding: 7px 10px 0;
    }

    .crossing-bus-window {
        height: 11px;
        border-radius: 2px;
        background: #ffffff;
    }

    .crossing-bus-wheel {
        position: absolute;
        bottom: 0;
        width: 13px;
        height: 7px;
        border-radius: 0 0 13px 13px;
        background: #050505;
    }

    .crossing-bus-wheel.left {
        left: 18px;
    }

    .crossing-bus-wheel.right {
        right: 18px;
    }

    @keyframes bus-cross-right {
        from {
            transform: translateX(-120px);
        }

        to {
            transform: translateX(calc(100vw + 120px));
        }
    }

    @keyframes bus-cross-left {
        from {
            transform: translateX(calc(100vw + 120px));
        }

        to {
            transform: translateX(-120px);
        }
    }

    .footer {
        flex: 0 0 auto;
        width: 100%;
        min-height: 32px;
        background: #e4e4e4;
        color: #101010;
        display: grid;
        grid-template-columns: auto minmax(0, 1fr) auto;
        align-items: center;
        gap: 20px;
        padding: 6px 24px;
        box-sizing: border-box;
        font-size: 12px;
        line-height: 1.2;
    }

    .footer p {
        margin: 0;
    }

    .last-updated {
        color: #404040;
        white-space: nowrap;
    }

    @media (max-width: 900px) {
        .top-bar {
            grid-template-columns: minmax(0, 1fr) auto;
            gap: 16px;
            padding: 8px 18px;
        }

        .header-status {
            grid-column: 2;
            justify-self: end;
            gap: 10px;
        }

        .header-wordmark {
            height: 28px;
            max-width: 220px;
        }

        .header-location {
            grid-column: 1 / -1;
            grid-row: 2;
            font-size: 22px;
        }

        .weather-chip {
            gap: 8px;
            padding: 5px 10px 6px;
        }

        .weather-icon {
            font-size: 22px;
        }

        .weather-divider {
            height: 18px;
        }

        .weather-detail {
            font-size: 9px;
        }

        .display {
            grid-template-columns: 36px minmax(0, 1fr);
        }

        .board-content {
            padding: 24px 24px 48px;
        }

        .board-grid {
            grid-template-columns: 1fr;
            gap: 30px;
        }

        .stop-panel + .stop-panel {
            border-left: 0;
            border-top: 2px solid rgba(5, 5, 5, 0.32);
            padding-left: 0;
            padding-top: 30px;
        }

        .stop-qr-cluster {
            bottom: 136px;
            gap: 8px;
        }

        .stop-qr {
            width: 78px;
            font-size: 9px;
        }

        .stop-qr span {
            width: 78px;
        }

        .stop-qr-secondary {
            width: 92px;
            font-size: 8px;
        }

        .stop-qr-secondary span {
            width: 92px;
        }

        .stop-qr img {
            width: 78px;
            height: 78px;
        }

        h1,
        h2 {
            align-items: flex-start;
            flex-direction: column;
            gap: 6px;
        }

        .heading-title,
        .heading-meta {
            align-items: flex-start;
            flex-wrap: wrap;
            justify-content: flex-start;
        }

        .next-route-grid {
            grid-template-columns: repeat(2, minmax(0, 1fr));
            padding-right: 88px;
        }

        .footer {
            align-items: flex-start;
            gap: 4px;
            grid-template-columns: auto 1fr;
        }

        .last-updated {
            grid-column: 2;
        }
    }
</style>
