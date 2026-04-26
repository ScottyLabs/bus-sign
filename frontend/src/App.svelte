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

    type NextRouteSummary = {
        route: string;
        destination: string;
        seconds: number;
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
    let crossingBuses: CrossingBus[] = [];
    let nextCrossingBusId = 0;

    const UC_DIRECTIONS = ["Towards Squirrel Hill", "From Downtown", "Heading East", "Toward Beeler St"];
    const TEPPER_DIRECTIONS = [
        "Toward Oakland (Univ. of Pitt)",
        "Toward Downtown",
        "Heading West",
        "To Craig St",
    ];
    const MAJOR_ROUTES = ["61A", "61B", "61C", "61D", "67", "28X"];
    const MAIN_LIST_SECONDS_LIMIT = 30 * 60;
    const NEXT_MAJOR_SECONDS_LIMIT = 60 * 60;
    const MAIN_LIST_ENTRY_LIMIT = 5;
    const NEXT_MAJOR_VISIBLE_LIMIT = 5;
    const API_BASE = import.meta.env.VITE_API_BASE || "";
    const CROSSING_BUS_COLORS = ["#bd1238", "#00a970", "#ffc627", "#007bbf"];
    const CROSSING_BUS_DURATION_MS = 20 * 1000;
    const STARTUP_BUS_COUNT = 5;
    const STARTUP_BUS_WINDOW_MS = 30 * 1000;
    const UC_STOP_URL =
        "https://realtime.portauthority.org/bustime/eta/eta.jsp?id=7117&showAllBusses=on";
    const TEP_STOP_URL =
        "https://realtime.portauthority.org/bustime/eta/eta.jsp?id=4407&showAllBusses=on";

    const normalizeRoute = (route: string) => route.trim().toUpperCase();

    const isMajorRoute = (route: string) => MAJOR_ROUTES.includes(normalizeRoute(route));

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

    const formatTimeRemaining = (seconds: number): string => {
        if (seconds < 60) return "NOW";
        return `${Math.ceil(seconds / 60)}m`;
    };

    const getHiddenMajorRouteSummaries = (
        entries: RouteInformation[],
        mainEntries: RouteInformation[],
    ): NextRouteSummary[] => {
        const visibleEntries = new Set(mainEntries);

        return entries
            .filter(
                (entry) =>
                    !visibleEntries.has(entry) &&
                    entry.arrivals[0] &&
                    isWithinNextMajorWindow(entry),
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
    };

    const getVisibleNextRoutes = (routes: NextRouteSummary[]) => {
        if (routes.length <= NEXT_MAJOR_VISIBLE_LIMIT) return routes;

        const start = (nextRouteIndex * NEXT_MAJOR_VISIBLE_LIMIT) % routes.length;
        return Array.from(
            { length: NEXT_MAJOR_VISIBLE_LIMIT },
            (_, offset) => routes[(start + offset) % routes.length],
        );
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

    const generateQrCodes = async () => {
        try {
            const options = {
                errorCorrectionLevel: "L" as const,
                margin: 1,
                width: 112,
                color: {
                    dark: "#ffffff",
                    light: "#050505",
                },
            };

            [qrCodeUC, qrCodeTep] = await Promise.all([
                QRCode.toDataURL(UC_STOP_URL, options),
                QRCode.toDataURL(TEP_STOP_URL, options),
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
    $: nextRoutesUC = getHiddenMajorRouteSummaries(entriesUC, mainEntriesUC);
    $: nextRoutesTep = getHiddenMajorRouteSummaries(entriesTep, mainEntriesTep);
    $: visibleNextRoutesUC = getVisibleNextRoutes(nextRoutesUC);
    $: visibleNextRoutesTep = getVisibleNextRoutes(nextRoutesTep);
    $: ucDirection = UC_DIRECTIONS[directionIndex % UC_DIRECTIONS.length];
    $: tepperDirection = TEPPER_DIRECTIONS[directionIndex % TEPPER_DIRECTIONS.length];

    onMount(() => {
        void refresh();
        void generateQrCodes();
        const refreshInterval = setInterval(refresh, 3_000);
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
                <img src="/scotty.svg" alt="Scotty Logo" class="header-logo" />
                <span>Carnegie Mellon University</span>
            </div>
            <div class="header-location">Forbes & Morewood Bus Stops</div>
            <div class="date-mark">{displayDate}<br />{displayClock}</div>
        </header>

        <div class="display">
            <aside class="pattern-rail" aria-hidden="true"></aside>

            <div class="board-content">
                <div class="board-grid">
                <section class="stop-panel" aria-labelledby="uc-side-heading">
                    <h1 id="uc-side-heading">
                        <span class="heading-title">
                            <span class="heading-main">
                                <span>UC Side</span>
                                <small>{ucDirection}</small>
                            </span>
                            <small class="stop-number">Stop #7117</small>
                        </span>
                        <span class="heading-meta">
                            <small>(1-2 minute walk)</small>
                        </span>
                    </h1>

                    <div class="arrival-list">
                        {#each mainEntriesUC as entry (entry.route + entry.destination)}
                            <BusTimeEntry
                                {...entry}
                                highlightStartSeconds={60}
                                highlightEndSeconds={120}
                            />
                        {:else}
                            <BusTimeEntry
                                route={"No buses"}
                                destination={"Check back soon"}
                                arrivals={[]}
                            />
                        {/each}
                    </div>

                    <div class="next-summary" aria-label="Next hidden major routes for UC Side">
                        <p>Next major routes</p>
                        <div class="next-route-grid">
                            {#if visibleNextRoutesUC.length > 0}
                                {#each visibleNextRoutesUC as entry (entry.route)}
                                    <div class="next-route">
                                        <strong>{entry.route}</strong>
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
                        <a class="stop-qr" href={UC_STOP_URL} target="_blank" rel="noreferrer">
                            <span>Take me on<br />the go!</span>
                            <img src={qrCodeUC} alt="QR code for live UC Side arrivals at stop 7117" />
                        </a>
                    {/if}
                </section>

                <section class="stop-panel" aria-labelledby="tepper-side-heading">
                    <h2 id="tepper-side-heading">
                        <span class="heading-title">
                            <span class="heading-main">
                                <span>Tepper Side</span>
                                <small>{tepperDirection}</small>
                            </span>
                            <small class="stop-number">Stop #4407</small>
                        </span>
                        <span class="heading-meta">
                            <small>(3-5 minute walk)</small>
                        </span>
                    </h2>

                    <div class="arrival-list">
                        {#each mainEntriesTep as entry (entry.route + entry.destination)}
                            <BusTimeEntry
                                {...entry}
                                highlightStartSeconds={60}
                                highlightEndSeconds={300}
                            />
                        {:else}
                            <BusTimeEntry
                                route={"No buses"}
                                destination={"Check back soon"}
                                arrivals={[]}
                            />
                        {/each}
                    </div>

                    <div class="next-summary" aria-label="Next hidden major routes for Tepper Side">
                        <p>Next major routes</p>
                        <div class="next-route-grid">
                            {#if visibleNextRoutesTep.length > 0}
                                {#each visibleNextRoutesTep as entry (entry.route)}
                                    <div class="next-route">
                                        <strong>{entry.route}</strong>
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
                        <a class="stop-qr" href={TEP_STOP_URL} target="_blank" rel="noreferrer">
                            <span>Take me on<br />the go!</span>
                            <img src={qrCodeTep} alt="QR code for live Tepper Side arrivals at stop 4407" />
                        </a>
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
        background: #030303;
        color: #f9f7ef;
    }

    .top-bar {
        min-height: 74px;
        background: #bd1238;
        color: #fff;
        display: grid;
        grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
        align-items: center;
        gap: 28px;
        padding: 10px 32px;
        box-sizing: border-box;
    }

    .brand-lockup {
        display: flex;
        align-items: center;
        gap: 14px;
        min-width: 0;
        font-size: 18px;
        line-height: 1;
        text-transform: uppercase;
    }

    .header-location {
        min-width: 0;
        font-size: clamp(24px, 2.4vw, 40px);
        line-height: 1;
        text-align: center;
        white-space: nowrap;
    }

    .date-mark {
        font-size: 22px;
        line-height: 1.05;
        text-align: right;
        white-space: nowrap;
    }

    .header-logo {
        width: auto;
        height: 44px;
        flex-shrink: 0;
    }

    .display {
        flex: 1 1 auto;
        min-height: 0;
        display: grid;
        grid-template-columns: 58px minmax(0, 1fr);
        background: #050505;
    }

    .pattern-rail {
        background:
            repeating-linear-gradient(25deg, transparent 0 12px, rgba(255, 198, 39, 0.9) 12px 14px, transparent 14px 26px),
            repeating-linear-gradient(150deg, transparent 0 14px, rgba(0, 123, 191, 0.9) 14px 16px, transparent 16px 28px),
            repeating-linear-gradient(115deg, transparent 0 8px, rgba(0, 169, 112, 0.9) 8px 10px, transparent 10px 18px),
            repeating-linear-gradient(65deg, transparent 0 10px, rgba(214, 18, 64, 0.9) 10px 12px, transparent 12px 22px),
            #111;
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
        padding-bottom: 132px;
    }

    .stop-panel + .stop-panel {
        border-left: 2px solid rgba(255, 255, 255, 0.32);
        padding-left: 42px;
    }

    h1,
    h2 {
        margin: 0 0 10px;
        font-size: clamp(28px, 3.1vw, 52px);
        line-height: 1;
        color: #ffffff;
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
        flex-direction: column;
        align-items: flex-start;
        gap: 2px;
    }

    .heading-main {
        min-width: 0;
        display: flex;
        align-items: baseline;
        gap: 12px;
    }

    .heading-main > span {
        flex: 0 0 auto;
    }

    .heading-meta {
        flex: 0 0 auto;
        justify-content: flex-end;
    }

    h1 small,
    h2 small {
        color: #cfcfcf;
        font-size: clamp(13px, 1.1vw, 18px);
        line-height: 1;
        white-space: nowrap;
    }

    .stop-number {
        color: #cfcfcf;
        font-size: clamp(13px, 1.1vw, 18px);
        font-style: italic;
        line-height: 1;
        text-align: left;
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
        color: #e5e5e5;
    }

    .next-summary p {
        margin: 0 0 8px;
        color: #a8a8a8;
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
        border-top: 1px solid rgba(255, 255, 255, 0.26);
        padding-top: 8px;
        display: grid;
        grid-template-columns: auto minmax(0, 1fr);
        gap: 2px 8px;
        align-items: baseline;
        color: #f5f5f5;
    }

    .next-route strong {
        color: #ffc627;
        font-size: 18px;
        line-height: 1;
    }

    .next-route span {
        color: #ffffff;
        font-size: 15px;
        line-height: 1;
        white-space: nowrap;
    }

    .next-route small {
        grid-column: 1 / -1;
        min-width: 0;
        color: #9ec9ec;
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
        color: #777;
    }

    .stop-qr {
        position: absolute;
        right: 0;
        bottom: 56px;
        z-index: 3;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 6px;
        width: 96px;
        color: #ffffff;
        font-size: 11px;
        line-height: 1.05;
        text-align: center;
        text-decoration: none;
        white-space: nowrap;
    }

    .stop-qr span {
        width: 96px;
    }

    .stop-qr img {
        display: block;
        width: 96px;
        height: 96px;
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
        border: 3px solid #050505;
        box-sizing: border-box;
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 6px;
        padding: 7px 10px 0;
    }

    .crossing-bus-window {
        height: 11px;
        border-radius: 2px;
        background: #050505;
    }

    .crossing-bus-wheel {
        position: absolute;
        bottom: 0;
        width: 13px;
        height: 7px;
        border-radius: 0 0 13px 13px;
        background: #ffffff;
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
        background: #1b1b1b;
        color: #efefe9;
        display: flex;
        justify-content: space-between;
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
        color: #cfcfcf;
        white-space: nowrap;
    }

    @media (max-width: 900px) {
        .top-bar {
            grid-template-columns: minmax(0, 1fr) auto;
            gap: 16px;
            padding: 8px 18px;
        }

        .header-location {
            grid-column: 1 / -1;
            grid-row: 2;
            font-size: 22px;
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
            border-top: 2px solid rgba(255, 255, 255, 0.32);
            padding-left: 0;
            padding-top: 30px;
        }

        .stop-qr {
            width: 78px;
            font-size: 9px;
            bottom: 46px;
        }

        .stop-qr span {
            width: 78px;
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
            flex-direction: column;
            gap: 4px;
        }
    }
</style>
