<script lang="ts">
    export let route: string;
    export let destination: string;
    export let arrivals: {
        bus_id: string;
        capacity: string;
        seconds: number;
    }[];
    export let routeColor: string | null = null;
    export let highlightStartSeconds: number | null = null;
    export let highlightEndSeconds: number | null = null;
    export let placeholder = false;

    const capacityLevels: Record<string, number> = {
        EMPTY: 0,
        HALF_EMPTY: 1,
        MEDIUM: 1,
        MANY_SEATS_AVAILABLE: 1,
        FEW_SEATS_AVAILABLE: 2,
        STANDING_ROOM_ONLY: 2,
        FULL: 3,
    };

    const capacityLabels: Record<number, string> = {
        0: "Empty",
        1: "Not busy",
        2: "Busy",
        3: "Crowded",
    };
    const capacityColors: Record<number, { active: string; inactive: string; label: string }> = {
        0: {
            active: "#b8f3c7",
            inactive: "#b8f3c7",
            label: "#16743c",
        },
        1: {
            active: "#ffc627",
            inactive: "#fff2a8",
            label: "#ffc627",
        },
        2: {
            active: "#e66f00",
            inactive: "#ffd3a6",
            label: "#e66f00",
        },
        3: {
            active: "#bd1238",
            inactive: "#f5b0bf",
            label: "#bd1238",
        },
    };

    const formatTimeRemaining = (seconds: number): string => {
        if (seconds < 60) return "NOW";
        return `${Math.ceil(seconds / 60)}m`;
    };

    const formatClockTime = (seconds: number): string => {
        const date = new Date(Date.now() + seconds * 1000);
        return date.toLocaleTimeString("en-US", {
            hour: "numeric",
            minute: "2-digit",
        });
    };

    const formatTitleCase = (value: string): string =>
        value
            .toLowerCase()
            .replace(/\b[a-z]/g, (letter) => letter.toUpperCase())
            .replace(/\bMc([a-z])/g, (_, letter: string) => `Mc${letter.toUpperCase()}`)
            .replace(/\bCcac\b/g, "CCAC");

    $: nextArrival = arrivals[0];
    $: timeDisplay = nextArrival ? formatTimeRemaining(nextArrival.seconds) : "--";
    $: isNow = nextArrival ? nextArrival.seconds < 60 : false;
    $: isWalkWindow =
        nextArrival && highlightStartSeconds !== null && highlightEndSeconds !== null
            ? nextArrival.seconds >= highlightStartSeconds &&
              nextArrival.seconds <= highlightEndSeconds
            : false;
    $: clockTime = nextArrival ? formatClockTime(nextArrival.seconds) : "";
    $: vehicleLabel = nextArrival?.bus_id || "No vehicle";
    $: destinationDisplay =
        route.trim().toUpperCase() === "28X" &&
        destination.toLowerCase().includes("airport")
            ? "(✈ PIT) Pittsburgh International Airport"
            : formatTitleCase(destination || vehicleLabel);
    $: destinationColor = routeColor;
    $: capacityLevel = nextArrival?.capacity
        ? capacityLevels[nextArrival.capacity.trim().toUpperCase()] || 0
        : 0;
    $: capacityColor = capacityColors[capacityLevel];
</script>

<article
    class="arrival-row"
    class:walk-window-row={isWalkWindow && !isNow}
    class:now-row={isNow}
    class:placeholder-row={placeholder}
    aria-hidden={placeholder}
>
    <div class="route-copy">
        <div class="route">
            <span style:color={routeColor}>{route}</span>
            {#if nextArrival?.bus_id}
                <small>{nextArrival.bus_id}</small>
            {/if}
        </div>
        <div class="destination" style:color={destinationColor}>
            {destinationDisplay}
        </div>
    </div>

    <div
        class="capacity-icons"
        aria-label="Capacity level {capacityLevel} of 3"
        style:--capacity-active={capacityColor.active}
        style:--capacity-inactive={capacityColor.inactive}
        style:--capacity-label={capacityColor.label}
    >
        <div class="person-row">
            {#each [1, 2, 3] as level}
                <span class:active={capacityLevel >= level} class="person-icon">
                    <span class="person-head"></span>
                    <span class="person-body"></span>
                </span>
            {/each}
        </div>
        <span class="capacity-label">{capacityLabels[capacityLevel]}</span>
    </div>

    <div class="time-copy">
        <div class="time" class:now={isNow}>{timeDisplay}</div>
        <div class="arrival-clock">{clockTime}</div>
    </div>
</article>

<style>
    .arrival-row {
        display: grid;
        grid-template-columns: minmax(0, 1fr) 96px 116px;
        align-items: center;
        gap: 20px;
        height: 106px;
        padding: 12px 0;
        box-sizing: border-box;
        border-top: 1px solid rgba(5, 5, 5, 0.2);
        color: #050505;
    }

    .walk-window-row {
        animation: walk-window-flash 3.2s ease-in-out infinite;
    }

    .now-row {
        animation: now-row-flash 2s ease-in-out infinite;
    }

    .arrival-row:last-child {
        border-bottom: 1px solid rgba(5, 5, 5, 0.2);
    }

    .placeholder-row {
        color: transparent;
    }

    .placeholder-row .route-copy,
    .placeholder-row .capacity-icons,
    .placeholder-row .time-copy {
        visibility: hidden;
    }

    .route-copy {
        min-width: 0;
    }

    .route {
        display: flex;
        align-items: baseline;
        gap: 10px;
        min-width: 0;
        font-size: clamp(28px, 3.1vw, 52px);
        line-height: 0.95;
        letter-spacing: 0;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .route span {
        min-width: 0;
        overflow: hidden;
        text-overflow: ellipsis;
        font-weight: 600;
    }

    .route small {
        flex: 0 0 auto;
        color: #404040;
        font-size: clamp(12px, 1vw, 18px);
        line-height: 1;
        white-space: nowrap;
    }

    .destination {
        margin-top: 5px;
        color: #272727;
        font-size: clamp(12px, 0.95vw, 17px);
        line-height: 1.1;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .time-copy {
        display: flex;
        flex-direction: column;
        align-items: flex-end;
        width: 116px;
        justify-self: end;
    }

    .capacity-icons {
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        gap: 5px;
        width: 96px;
        justify-self: start;
        transform: translateX(-44px);
    }

    .person-row {
        display: grid;
        grid-template-columns: repeat(3, 22px);
        justify-content: center;
        align-items: end;
        gap: 8px;
        min-height: 42px;
    }

    .person-icon {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 3px;
        opacity: 0.45;
    }

    .person-icon.active {
        opacity: 1;
    }

    .person-head {
        width: 14px;
        height: 14px;
        border-radius: 50%;
        background: var(--capacity-inactive);
    }

    .person-body {
        width: 22px;
        height: 25px;
        border-radius: 8px 8px 3px 3px;
        background: var(--capacity-inactive);
    }

    .person-icon.active .person-head,
    .person-icon.active .person-body {
        background: var(--capacity-active);
    }

    .capacity-label {
        color: var(--capacity-label);
        font-size: 11px;
        line-height: 1;
        min-height: 11px;
        text-align: center;
        text-transform: uppercase;
        white-space: nowrap;
    }

    .time {
        color: #050505;
        font-size: clamp(28px, 3vw, 50px);
        line-height: 0.95;
        white-space: nowrap;
        letter-spacing: 0;
    }

    .arrival-clock {
        margin-top: 5px;
        color: #050505;
        font-size: clamp(12px, 1vw, 18px);
        line-height: 1;
        white-space: nowrap;
    }

    .now {
        animation: pulse 1.5s ease-in-out infinite;
    }

    @keyframes walk-window-flash {
        0%,
        100% {
            background: #fffdf1;
        }
        50% {
            background: #fff8d8;
        }
    }

    @keyframes now-row-flash {
        0%,
        100% {
            background: #f9d4db;
        }
        50% {
            background: #efb5bf;
        }
    }

    @media (max-width: 900px) {
        .arrival-row {
            grid-template-columns: minmax(0, 1fr) 74px 92px;
            gap: 14px;
            height: 86px;
        }

        .capacity-icons {
            width: 74px;
            justify-self: start;
            transform: translateX(-28px);
        }

        .person-row {
            grid-template-columns: repeat(3, 17px);
            gap: 5px;
            min-height: 33px;
        }

        .person-head {
            width: 10px;
            height: 10px;
        }

        .person-body {
            width: 17px;
            height: 20px;
        }

        .time-copy {
            width: 92px;
        }
    }
</style>
