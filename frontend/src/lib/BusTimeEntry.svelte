<script lang="ts">
    export let route: string;
    export let destination: string;
    export let arrivals: {
        bus_id: string;
        capacity: string;
        seconds: number;
    }[];
    export let highlightStartSeconds: number | null = null;
    export let highlightEndSeconds: number | null = null;

    const capacityLevels: Record<string, number> = {
        EMPTY: 0,
        HALF_EMPTY: 2,
        FULL: 3,
    };

    const capacityLabels: Record<number, string> = {
        0: "Empty",
        1: "Not busy",
        2: "Busy",
        3: "Crowded",
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
            .replace(/\bMc([a-z])/g, (_, letter: string) => `Mc${letter.toUpperCase()}`);

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
    $: capacityLevel = nextArrival?.capacity
        ? capacityLevels[nextArrival.capacity] || 0
        : 0;
</script>

<article class="arrival-row">
    <div class="mode-icon" aria-hidden="true">
        <span class="bus-window"></span>
        <span class="bus-body"></span>
        <span class="bus-wheel left-wheel"></span>
        <span class="bus-wheel right-wheel"></span>
    </div>

    <div class="route-copy">
        <div class="route">
            <span>{route}</span>
            {#if nextArrival?.bus_id}
                <small>{nextArrival.bus_id}</small>
            {/if}
        </div>
        <div class="destination">
            {destinationDisplay}
        </div>
    </div>

    <div class="capacity-icons" aria-label="Capacity level {capacityLevel} of 3">
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
        <div class="time" class:now={isNow} class:walk-window={isWalkWindow}>{timeDisplay}</div>
        <div class="arrival-clock">{clockTime}</div>
    </div>
</article>

<style>
    .arrival-row {
        display: grid;
        grid-template-columns: 66px minmax(0, 1fr) 96px 116px;
        align-items: center;
        gap: 20px;
        min-height: 94px;
        padding: 12px 0;
        box-sizing: border-box;
        border-top: 1px solid rgba(255, 255, 255, 0.2);
        color: #ffffff;
    }

    .arrival-row:last-child {
        border-bottom: 1px solid rgba(255, 255, 255, 0.2);
    }

    .mode-icon {
        position: relative;
        width: 48px;
        height: 48px;
        display: grid;
        place-items: center;
        border: 4px solid #f3b7c9;
        border-radius: 50%;
        box-sizing: border-box;
    }

    .bus-window,
    .bus-body,
    .bus-wheel {
        position: absolute;
        display: block;
        box-sizing: border-box;
    }

    .bus-body {
        width: 24px;
        height: 23px;
        border: 3px solid #f3b7c9;
        border-radius: 3px;
        top: 10px;
        left: 8px;
    }

    .bus-window {
        width: 20px;
        height: 8px;
        border: 2px solid #f3b7c9;
        top: 14px;
        left: 10px;
    }

    .bus-wheel {
        width: 5px;
        height: 5px;
        background: #f3b7c9;
        border-radius: 50%;
        bottom: 11px;
    }

    .left-wheel {
        left: 14px;
    }

    .right-wheel {
        right: 14px;
    }

    .route-copy {
        min-width: 0;
    }

    .route {
        display: flex;
        align-items: baseline;
        gap: 10px;
        min-width: 0;
        font-size: clamp(30px, 3.4vw, 58px);
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
    }

    .route small {
        flex: 0 0 auto;
        color: #cfcfcf;
        font-size: clamp(12px, 1vw, 18px);
        line-height: 1;
        white-space: nowrap;
    }

    .destination {
        margin-top: 5px;
        color: #d8d8d8;
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
        background: #777;
    }

    .person-body {
        width: 22px;
        height: 25px;
        border-radius: 8px 8px 3px 3px;
        background: #777;
    }

    .person-icon.active .person-head,
    .person-icon.active .person-body {
        background: #f2f2f2;
    }

    .capacity-label {
        color: #d7d7d7;
        font-size: 11px;
        line-height: 1;
        min-height: 11px;
        text-align: center;
        text-transform: uppercase;
        white-space: nowrap;
    }

    .time {
        color: #ffffff;
        font-size: clamp(30px, 3.2vw, 56px);
        line-height: 0.95;
        white-space: nowrap;
        letter-spacing: 0;
    }

    .arrival-clock {
        margin-top: 5px;
        color: #ffffff;
        font-size: clamp(12px, 1vw, 18px);
        line-height: 1;
        white-space: nowrap;
    }

    .now {
        color: #82d7a4;
        animation: pulse 1.5s ease-in-out infinite;
    }

    .walk-window {
        color: #ffc627;
    }

    @media (max-width: 900px) {
        .arrival-row {
            grid-template-columns: 52px minmax(0, 1fr) 74px 92px;
            gap: 14px;
            min-height: 78px;
        }

        .mode-icon {
            width: 42px;
            height: 42px;
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
