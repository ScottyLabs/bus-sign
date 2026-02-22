<script lang="ts">
    export let route: string;
    export let destination: string;
    export let arrivals: {
        bus_id: string;
        capacity: string;
        seconds: number;
    }[];

    const routeColors: Record<string, string> = {
        "61A": "#e74c3c",
        "61B": "#3498db",
        "61C": "#2ecc71",
        "61D": "#f39c12",
        "67": "#9b59b6",
        "58": "#1abc9c",
    };

    const capacityInfo: Record<string, { label: string; color: string }> = {
        EMPTY: { label: "Empty", color: "#2ecc71" },
        HALF_EMPTY: { label: "Some seats", color: "#f1c40f" },
        FULL: { label: "Full", color: "#e74c3c" },
    };

    const formatTime = (seconds: number): string => {
        if (seconds < 30) return "Approaching";
        const minutes = Math.ceil(seconds / 60);
        return `${minutes}`;
    };

    $: nextArrival = arrivals[0];
    $: timeDisplay = nextArrival ? formatTime(nextArrival.seconds) : "N/A";
    $: isApproaching = timeDisplay === "Approaching";
    $: upcomingTimes = arrivals
        .slice(1, 3)
        .map((a) => formatTime(a.seconds))
        .join(", ");
    $: badgeColor = routeColors[route] || "#6c757d";
    $: capacity = nextArrival?.capacity
        ? capacityInfo[nextArrival.capacity]
        : null;
</script>

<div class="rounded-box container">
    <div class="stack left">
        <div class="route" style="color: {badgeColor}">
            {route}
        </div>
        <div>
            To {destination.toUpperCase()}
            {#if capacity}
                <span class="capacity">({capacity.label})</span>
            {/if}
        </div>
    </div>
    <div class="stack right">
        <div class="time" class:approaching={isApproaching}>
            {#if timeDisplay === "Approaching"}
                {timeDisplay}
            {:else}
                {timeDisplay} MIN
            {/if}
        </div>
        {#if upcomingTimes.length > 0}
            <div>
                Next bus in {upcomingTimes} min
            </div>
        {/if}
    </div>
</div>

<style>
    .rounded-box {
        border-radius: 12px;
        padding: 12px 16px;
        background-color: #f5f5f5;
        border: 1px solid #e0e0e0;
        min-width: 100%;
        min-height: 80px;
        box-sizing: border-box;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
    }

    .route {
        font-size: 40px;
    }

    .time-row {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .time {
        font-size: 32px;
    }

    .approaching {
        color: #2ecc71;
        animation: pulse 1.5s ease-in-out infinite;
    }

    .capacity {
        font-style: italic;
        font-weight: normal;
    }
</style>
