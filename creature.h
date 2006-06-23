/*
   
    Copyright (c) 2006 Florian Wesch <fw@dividuum.de>. All Rights Reserved.
    
    This program is free software; you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation; either version 2 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.
    
    You should have received a copy of the GNU General Public License along
    with this program; if not, write to the Free Software Foundation, Inc.,
    59 Temple Place - Suite 330, Boston, MA  02111-1307, USA

*/

#ifndef CREATURE_H
#define CREATURE_H

#include "path.h"
#include "player.h"
#include "server.h"

#define MAXCREATURES       256

#define CREATURE_COLORS     16
#define CREATURE_TYPES       4
#define CREATURE_DIRECTIONS 32
#define CREATURE_ANIMS       2

typedef struct creature_s {
    int         x;
    int         y;
    int         dir;
    int         type;
    int         food;
    int         health;
    player_t   *player;
    int         target;
    pathnode_t *path;
    int         convert_food;
    int         convert_type;
    int         spawn_food;
    enum {
        CREATURE_IDLE,
        CREATURE_WALK,
        CREATURE_HEAL,
        CREATURE_EAT,
        CREATURE_ATTACK,
        CREATURE_CONVERT,
        CREATURE_SPAWN,
        CREATURE_FEED,
    } state;

    int  age_action_deltas;
    int  last_state_change;
    int  spawn_time;

    int  network_health;
    int  network_food;
    int  network_x;
    int  network_y;
    int  network_dir;
    
    char message[9];
    int  last_msg_set;

    unsigned char dirtymask;
#define CREATURE_DIRTY_ALIVE   (1 << 0) 
#define CREATURE_DIRTY_POS     (1 << 1) 
#define CREATURE_DIRTY_TYPE    (1 << 2)
#define CREATURE_DIRTY_FOOD    (1 << 3)
#define CREATURE_DIRTY_HEALTH  (1 << 4)
#define CREATURE_DIRTY_STATE   (1 << 5)
#define CREATURE_DIRTY_TARGET  (1 << 6)
#define CREATURE_DIRTY_MESSAGE (1 << 7)

#define CREATURE_DIRTY_ALL         0xFF
#define CREATURE_DIRTY_NONE        0x00
} creature_t;

int         creature_num(const creature_t *creature);
creature_t *creature_by_num(int creature_num);
creature_t *creature_spawn(player_t *player, int x, int y, int type, int points);
void        creature_kill(creature_t *creature, creature_t *killer);

int         creature_set_path(creature_t *creature, int x, int y);
int         creature_set_target(creature_t *creature, int target);
int         creature_set_state(creature_t *creature, int state);
int         creature_set_conversion_type(creature_t *creature, int type);
void        creature_set_message(creature_t *creature, const char *message);

creature_t *creature_nearest_enemy(const creature_t *reference, int *distptr);
int         creature_max_health(const creature_t *creature);
int         creature_speed(const creature_t *creature);
int         creature_dist(const creature_t *a, const creature_t *b);
int         creature_food_on_tile(const creature_t *creature);
int         creature_max_food(const creature_t *creature);
int         creature_hitpoints(const creature_t *creature);
int         creature_attack_distance(const creature_t *creature);

void        creature_kill_all_players_creatures(player_t *player);
int         creature_king_player();
void        creature_moveall(int delta);
void        creature_draw();

/* Network */
void        creature_send_initial_update(client_t *client);
void        creature_to_network(creature_t *creature, int dirtymask, client_t *client);

void        creature_init();
void        creature_shutdown();

#endif
