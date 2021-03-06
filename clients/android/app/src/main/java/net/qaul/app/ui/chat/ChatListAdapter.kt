package net.qaul.app.ui.chat

import android.util.Log
import android.view.View
import android.view.ViewGroup
import androidx.fragment.app.Fragment
import androidx.fragment.app.FragmentManager
import androidx.fragment.app.FragmentStatePagerAdapter
import androidx.fragment.app.FragmentTransaction
import androidx.recyclerview.widget.RecyclerView
import kotlinx.android.synthetic.main.item_chat_room.view.*
import net.qaul.app.R
import net.qaul.app.ffi.models.ChatRoom
import net.qaul.app.util.inflate

class ChatListAdapter(private val rooms: MutableList<ChatRoom>, private val fragMan: FragmentManager)
    : RecyclerView.Adapter<ChatListAdapter.RoomHolder>() {

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): RoomHolder {
        val inflated = parent.inflate(R.layout.item_chat_room, false)
        return RoomHolder(inflated, fragMan)
    }

    override fun getItemCount() = rooms.size

    override fun onBindViewHolder(holder: RoomHolder, position: Int) {
        holder.bindRoom(rooms[position])
    }

    class RoomHolder(v: View, private val man: FragmentManager)
        : RecyclerView.ViewHolder(v), View.OnClickListener {
        private var view: View = v
        var room: ChatRoom? = null

        init {
            v.setOnClickListener(this)
        }

        fun bindRoom(room: ChatRoom) {
            this.room = room

            // Then set the UI state
            view.chatroom_list_item_name.text = room.name!!
            view.chatroom_list_item_timestamp.text = room.last_message!!
            view.chatroom_list_item_unread_count.text = room.unread.toString()
        }

        override fun onClick(v: View?) {
            ChatRoomFragment(room!!).transitionInto(man)
        }
    }
}
