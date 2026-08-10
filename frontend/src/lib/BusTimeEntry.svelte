<script lang="ts">
    import Plane from "~icons/lucide/plane";

    export let route: string;
    export let destination: string;
    export let arrivals: {
        bus_id: string;
        capacity: string;
        seconds: number;
    }[];
    export let scheduled: boolean = false;

    type CapacityInfo = {
        label: string;
        icon: string;
    };

    const capacityInfo: Record<string, CapacityInfo> = {
        EMPTY: { label: "Empty", icon: "/empty.svg" },
        HALF_EMPTY: { label: "Moderate", icon: "/moderate.svg" },
        FULL: { label: "Busy", icon: "/busy.svg" },
    };

    const formatMinutes = (seconds: number): string => {
        if (seconds < 60) return "NOW";
        const minutes = Math.ceil(seconds / 60);
        return `${minutes} min`;
    };

    const formatClock = (seconds: number): string => {
        const arrival = new Date(Date.now() + seconds * 1000);
        return arrival.toLocaleTimeString("en-US", {
            timeZone: "America/New_York",
            hour: "numeric",
            minute: "2-digit",
        });
    };

    $: nextArrival = arrivals[0];
    $: seconds = nextArrival?.seconds ?? Infinity;
    $: timeDisplay = nextArrival ? formatMinutes(seconds) : "N/A";
    $: clockDisplay = nextArrival ? formatClock(seconds) : "";
    $: capacity = nextArrival?.capacity
        ? capacityInfo[nextArrival.capacity]
        : null;

    $: variant =
        seconds < 60
            ? "coming"
            : seconds < 180
              ? "soon"
              : scheduled
                ? "delayed"
                : "default";

    $: isAirport = route === "28X";

    $: routeColor = scheduled
        ? "text-gray"
        : isAirport
          ? "text-blue"
          : "text-black";

    $: timeColor =
        scheduled && variant === "coming"
            ? "text-black"
            : scheduled
              ? "text-gray"
              : variant === "coming"
                ? "text-red"
                : "text-black";

    $: rowClass =
        !scheduled && variant === "coming"
            ? "bg-red/10"
            : !scheduled && variant === "soon"
              ? "bg-yellow/20"
              : "bg-transparent";

    $: dividerClass =
        !scheduled && variant === "coming"
            ? "border-b-3 border-red"
            : !scheduled && variant === "soon"
              ? "border-b-3 border-yellow"
              : "border-b-2 border-light-gray";

    $: insetClass =
        !scheduled && (variant === "coming" || variant === "soon")
            ? "px-4"
            : "mx-4";
</script>

<div
    class="self-stretch pt-3 pb-1.5 transition-colors duration-500 {insetClass} {rowClass} {dividerClass}"
>
    <div class="self-stretch flex items-center">
        <div class="w-3/4 flex justify-between items-center gap-x-4 min-w-0">
            <div class="flex flex-1 flex-col justify-start items-start min-w-0">
                <div
                    class="inline-flex justify-start items-center {scheduled
                        ? 'gap-4'
                        : isAirport
                          ? 'gap-3'
                          : ''}"
                >
                    <div class="text-5xl font-semibold {routeColor}">
                        {route}
                    </div>
                    {#if isAirport && !scheduled}
                        <Plane class="size-8 shrink-0 {routeColor}" />
                    {/if}
                    {#if scheduled}
                        <div
                            class="px-2.5 py-1 bg-light-gray text-gray text-xs font-bold"
                        >
                            Scheduled
                        </div>
                    {/if}
                </div>
                <div class="w-full truncate text-xl font-semibold {routeColor}">
                    {destination}
                </div>
            </div>

            <div class="w-14 flex flex-col items-center gap-1.5 pt-2.5">
                {#if capacity}
                    <img src={capacity.icon} alt="" class="h-9 w-auto" />
                    <div class="text-gray">
                        {capacity.label}
                    </div>
                {/if}
            </div>
        </div>

        <div class="w-1/4 flex flex-col justify-center items-end">
            <div class="text-4xl/14 font-semibold {timeColor}">
                {timeDisplay}
            </div>
            {#if clockDisplay}
                <div
                    class="text-right text-xl font-semibold leading-5 {timeColor}"
                >
                    {clockDisplay}
                </div>
            {/if}
        </div>
    </div>
</div>
