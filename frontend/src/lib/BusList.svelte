<script lang="ts">
    import type { Component } from "svelte";
    import ArrowLeft from "~icons/lucide/arrow-left";
    import ArrowRight from "~icons/lucide/arrow-right";
    import BusTimeEntry from "./BusTimeEntry.svelte";

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

    export let title: string;
    export let direction: "inbound" | "outbound";
    export let stopId: string;
    export let walkMins: number;
    export let near: string;
    export let nextMajorRoutes: { route: string; minutes: number }[];
    export let entries: RouteInformation[];

    const arrows: Record<"inbound" | "outbound", Component> = {
        inbound: ArrowLeft,
        outbound: ArrowRight,
    };

    $: Arrow = arrows[direction];
</script>

<div class="flex flex-1 min-w-0 flex-col gap-9">
    <div class="self-stretch flex flex-col justify-start items-start gap-2.5">
        <div class="self-stretch inline-flex justify-between items-start">
            <div class="inline-flex flex-col justify-start items-start">
                <div class="inline-flex justify-start items-center gap-3">
                    <div
                        class="justify-start text-black text-5xl font-bold leading-[56px]"
                    >
                        {title}
                    </div>
                    <Arrow class="size-9 shrink-0" />
                </div>
                <div class="flex justify-start items-center gap-2">
                    <div
                        class="justify-start text-gray text-xl font-semibold leading-7"
                    >
                        Stop #{stopId}
                    </div>
                    <div class="w-[3px] h-7 shrink-0 bg-light-gray"></div>
                    <div class="justify-start text-xl font-semibold leading-7">
                        <span class="text-gray"
                            >{walkMins} mins walk near
                        </span>{" "}<span class="text-black">{near}</span>
                    </div>
                </div>
            </div>
            <div class="inline-flex flex-col justify-start items-end gap-1">
                <div class="justify-start text-black text-xl font-semibold">
                    Next major routes
                </div>
                <div
                    class="text-right justify-start text-gray text-xl font-medium leading-7"
                >
                    {#each nextMajorRoutes as major, i}
                        {#if i > 0}<br />{/if}{major.route} in {major.minutes} min
                    {/each}
                </div>
            </div>
        </div>
        <div class="self-stretch h-1 bg-red"></div>
    </div>
    <div
        class="self-stretch flex flex-col justify-start items-start gap-5 overflow-y-auto min-h-0"
    >
        {#each entries as entry}
            <BusTimeEntry
                route={entry.route}
                destination={entry.destination}
                arrivals={entry.arrivals}
                scheduled={entry.scheduled ?? false}
            />
        {/each}
    </div>
</div>
